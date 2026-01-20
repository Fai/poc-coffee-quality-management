import * as cdk from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as rds from 'aws-cdk-lib/aws-rds';
import * as ecs from 'aws-cdk-lib/aws-ecs';
import * as ecr from 'aws-cdk-lib/aws-ecr';
import * as elbv2 from 'aws-cdk-lib/aws-elasticloadbalancingv2';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import * as apigateway from 'aws-cdk-lib/aws-apigatewayv2';
import * as apigatewayIntegrations from 'aws-cdk-lib/aws-apigatewayv2-integrations';
import * as s3 from 'aws-cdk-lib/aws-s3';
import * as logs from 'aws-cdk-lib/aws-logs';
import * as budgets from 'aws-cdk-lib/aws-budgets';
import { Construct } from 'constructs';

export interface CoffeeQMStackProps extends cdk.StackProps {
  environment: string;
  costCenter: string;
  monthlyBudget: number;
  useFargateSpot: boolean;
  dbInstanceClass: string;
}

export class CoffeeQMStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: CoffeeQMStackProps) {
    super(scope, id, props);

    const project = 'CoffeeQualityManagement';

    // Apply tags to ALL resources in this stack
    cdk.Tags.of(this).add('Project', project);
    cdk.Tags.of(this).add('Environment', props.environment);
    cdk.Tags.of(this).add('CostCenter', props.costCenter);
    cdk.Tags.of(this).add('ManagedBy', 'cdk');

    // ========================================================================
    // VPC (No NAT Gateway for cost optimization)
    // ========================================================================
    const vpc = new ec2.Vpc(this, 'VPC', {
      maxAzs: 2,
      natGateways: 0, // Cost saving: no NAT
      subnetConfiguration: [
        { name: 'Public', subnetType: ec2.SubnetType.PUBLIC, cidrMask: 24 },
      ],
    });

    // ========================================================================
    // DATABASE - RDS PostgreSQL
    // ========================================================================
    const dbSecurityGroup = new ec2.SecurityGroup(this, 'DBSecurityGroup', {
      vpc,
      description: 'Database security group',
    });

    const database = new rds.DatabaseInstance(this, 'Database', {
      engine: rds.DatabaseInstanceEngine.postgres({ version: rds.PostgresEngineVersion.VER_15 }),
      instanceType: new ec2.InstanceType(props.dbInstanceClass),
      vpc,
      vpcSubnets: { subnetType: ec2.SubnetType.PUBLIC },
      securityGroups: [dbSecurityGroup],
      databaseName: 'coffee_qm',
      credentials: rds.Credentials.fromGeneratedSecret('cqm_admin'),
      allocatedStorage: 20,
      maxAllocatedStorage: 50,
      backupRetention: cdk.Duration.days(1),
      deleteAutomatedBackups: true,
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      publiclyAccessible: false,
    });

    // ========================================================================
    // ECS CLUSTER with Fargate Spot
    // ========================================================================
    const cluster = new ecs.Cluster(this, 'Cluster', {
      vpc,
      containerInsights: false, // Cost saving
    });

    // ECR Repository
    const repository = new ecr.Repository(this, 'BackendRepo', {
      repositoryName: `${project.toLowerCase()}-backend`,
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      lifecycleRules: [{ maxImageCount: 3 }],
    });

    // Task Definition
    const taskDef = new ecs.FargateTaskDefinition(this, 'TaskDef', {
      memoryLimitMiB: 512,
      cpu: 256,
    });

    const container = taskDef.addContainer('backend', {
      image: ecs.ContainerImage.fromEcrRepository(repository, 'latest'),
      logging: ecs.LogDrivers.awsLogs({
        streamPrefix: 'backend',
        logRetention: logs.RetentionDays.ONE_WEEK,
      }),
      environment: {
        CQM_ENVIRONMENT: props.environment,
      },
      secrets: {
        CQM__DATABASE__URL: ecs.Secret.fromSecretsManager(database.secret!, 'connectionString'),
      },
    });
    container.addPortMappings({ containerPort: 3000 });

    // ALB
    const alb = new elbv2.ApplicationLoadBalancer(this, 'ALB', {
      vpc,
      internetFacing: true,
    });

    const listener = alb.addListener('Listener', { port: 80 });

    // ECS Service with Fargate Spot
    const service = new ecs.FargateService(this, 'Service', {
      cluster,
      taskDefinition: taskDef,
      desiredCount: 1,
      assignPublicIp: true,
      capacityProviderStrategies: props.useFargateSpot
        ? [{ capacityProvider: 'FARGATE_SPOT', weight: 1 }]
        : [{ capacityProvider: 'FARGATE', weight: 1 }],
    });

    listener.addTargets('ECS', {
      port: 3000,
      targets: [service],
      healthCheck: { path: '/api/health', interval: cdk.Duration.seconds(60) },
    });

    // Allow ECS to connect to DB
    database.connections.allowFrom(service, ec2.Port.tcp(5432));

    // ========================================================================
    // AI - Lambda + API Gateway (pay-per-request)
    // ========================================================================
    const aiBucket = new s3.Bucket(this, 'AIImagesBucket', {
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      autoDeleteObjects: true,
      lifecycleRules: [{ expiration: cdk.Duration.days(7) }],
    });

    const aiLambda = new lambda.Function(this, 'DefectDetection', {
      runtime: lambda.Runtime.PYTHON_3_11,
      handler: 'index.handler',
      code: lambda.Code.fromInline(`
import json
import boto3
import base64
import os
from datetime import datetime

s3 = boto3.client('s3')

def handler(event, context):
    body = json.loads(event.get('body', '{}'))
    image_b64 = body.get('image_base64', '')
    
    if image_b64:
        key = f"uploads/{datetime.utcnow().strftime('%Y%m%d%H%M%S')}.jpg"
        s3.put_object(Bucket=os.environ['BUCKET_NAME'], Key=key, Body=base64.b64decode(image_b64))
    
    return {
        'statusCode': 200,
        'headers': {'Content-Type': 'application/json'},
        'body': json.dumps({
            'request_id': context.aws_request_id,
            'detected_beans': 100,
            'category1_count': 2,
            'category2_count': 5,
            'confidence': 0.92,
            'suggested_grade': 'Specialty'
        })
    }
`),
      timeout: cdk.Duration.seconds(60),
      memorySize: 256,
      environment: { BUCKET_NAME: aiBucket.bucketName },
    });

    aiBucket.grantReadWrite(aiLambda);

    const aiApi = new apigateway.HttpApi(this, 'AIApi', {
      apiName: `${project}-${props.environment}-ai`,
    });

    aiApi.addRoutes({
      path: '/detect',
      methods: [apigateway.HttpMethod.POST],
      integration: new apigatewayIntegrations.HttpLambdaIntegration('LambdaIntegration', aiLambda),
    });

    // ========================================================================
    // COST TRACKING - Budget Alert
    // ========================================================================
    new budgets.CfnBudget(this, 'ProjectBudget', {
      budget: {
        budgetName: `${project}-${props.environment}-monthly`,
        budgetType: 'COST',
        timeUnit: 'MONTHLY',
        budgetLimit: { amount: props.monthlyBudget, unit: 'USD' },
        costFilters: { TagKeyValue: [`user:Project$${project}`] },
      },
      notificationsWithSubscribers: [
        {
          notification: {
            notificationType: 'ACTUAL',
            comparisonOperator: 'GREATER_THAN',
            threshold: 80,
            thresholdType: 'PERCENTAGE',
          },
          subscribers: [{ subscriptionType: 'SNS', address: '' }], // Add SNS topic ARN
        },
      ],
    });

    // ========================================================================
    // OUTPUTS
    // ========================================================================
    new cdk.CfnOutput(this, 'ApiUrl', { value: `http://${alb.loadBalancerDnsName}/api` });
    new cdk.CfnOutput(this, 'AIEndpoint', { value: `${aiApi.apiEndpoint}/detect` });
    new cdk.CfnOutput(this, 'ECRRepository', { value: repository.repositoryUri });
    new cdk.CfnOutput(this, 'DatabaseEndpoint', { value: database.instanceEndpoint.hostname });
    new cdk.CfnOutput(this, 'CostFilter', { value: `Project=${project}, Environment=${props.environment}` });
  }
}
