import * as cdk from 'aws-cdk-lib';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import * as apigateway from 'aws-cdk-lib/aws-apigatewayv2';
import * as apigatewayIntegrations from 'aws-cdk-lib/aws-apigatewayv2-integrations';
import * as s3 from 'aws-cdk-lib/aws-s3';
import * as ecr from 'aws-cdk-lib/aws-ecr';
import * as codebuild from 'aws-cdk-lib/aws-codebuild';
import { Construct } from 'constructs';

export interface CoffeeAIStackProps extends cdk.StackProps {
  environment: string;
}

export class CoffeeAIStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: CoffeeAIStackProps) {
    super(scope, id, props);

    cdk.Tags.of(this).add('Project', 'CoffeeQualityManagement');
    cdk.Tags.of(this).add('Environment', props.environment);

    // S3 buckets
    const imagesBucket = new s3.Bucket(this, 'ImagesBucket', {
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      autoDeleteObjects: true,
      cors: [{
        allowedMethods: [s3.HttpMethods.GET, s3.HttpMethods.PUT, s3.HttpMethods.POST],
        allowedOrigins: ['*'],
        allowedHeaders: ['*'],
      }],
    });

    const frontendBucket = new s3.Bucket(this, 'FrontendBucket', {
      websiteIndexDocument: 'index.html',
      websiteErrorDocument: 'index.html',
      publicReadAccess: true,
      blockPublicAccess: s3.BlockPublicAccess.BLOCK_ACLS,
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      autoDeleteObjects: true,
    });

    // ECR Repository
    const ecrRepo = new ecr.Repository(this, 'AIRepo', {
      repositoryName: 'coffee-qm-ai',
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      emptyOnDelete: true,
    });

    // CodeBuild project
    const buildProject = new codebuild.Project(this, 'AIBuildProject', {
      projectName: 'coffee-qm-ai-build',
      environment: {
        buildImage: codebuild.LinuxBuildImage.STANDARD_7_0,
        privileged: true,
        computeType: codebuild.ComputeType.MEDIUM,
      },
      environmentVariables: {
        ECR_REPO_URI: { value: ecrRepo.repositoryUri },
        AWS_ACCOUNT_ID: { value: this.account },
        AWS_REGION: { value: this.region },
        BUCKET_NAME: { value: imagesBucket.bucketName },
      },
      buildSpec: codebuild.BuildSpec.fromObject({
        version: '0.2',
        phases: {
          pre_build: {
            commands: [
              'aws ecr get-login-password --region $AWS_REGION | docker login --username AWS --password-stdin $AWS_ACCOUNT_ID.dkr.ecr.$AWS_REGION.amazonaws.com',
            ],
          },
          build: {
            commands: [
              // Create Dockerfile
              `cat > Dockerfile << 'EOF'
FROM public.ecr.aws/lambda/python:3.11
COPY requirements.txt \${LAMBDA_TASK_ROOT}/
RUN pip install --no-cache-dir -r \${LAMBDA_TASK_ROOT}/requirements.txt
RUN python -c "from transformers import pipeline; pipeline('image-classification', model='everycoffee/autotrain-coffee-bean-quality-97496146930')"
COPY lambda_handler.py \${LAMBDA_TASK_ROOT}/
CMD ["lambda_handler.handler"]
EOF`,
              // Create requirements.txt
              `cat > requirements.txt << 'EOF'
boto3>=1.34.0
transformers>=4.36.0
torch>=2.1.0
Pillow>=10.0.0
EOF`,
              // Create lambda_handler.py
              `cat > lambda_handler.py << 'EOF'
import json, boto3, base64, os, uuid
from datetime import datetime
from io import BytesIO
s3 = boto3.client("s3")
BUCKET = os.environ.get("BUCKET_NAME")
MODEL = "everycoffee/autotrain-coffee-bean-quality-97496146930"
_clf = None
def get_clf():
    global _clf
    if _clf is None:
        from transformers import pipeline
        _clf = pipeline("image-classification", model=MODEL)
    return _clf
def handler(event, context):
    try:
        body = json.loads(event.get("body", "{}"))
        img_b64 = body.get("image_base64")
        if not img_b64:
            return {"statusCode": 400, "headers": {"Content-Type": "application/json", "Access-Control-Allow-Origin": "*"}, "body": json.dumps({"error": "Missing image_base64"})}
        rid = f"det-{datetime.utcnow().strftime('%Y%m%d%H%M%S')}-{str(uuid.uuid4())[:8]}"
        t0 = datetime.utcnow()
        img_bytes = base64.b64decode(img_b64)
        key = f"uploads/{rid}.jpg"
        s3.put_object(Bucket=BUCKET, Key=key, Body=img_bytes, ContentType="image/jpeg")
        from PIL import Image
        img = Image.open(BytesIO(img_bytes)).convert("RGB")
        res = get_clf()(img)
        d_score = next((r["score"] for r in res if r["label"].lower() == "defect"), 0.0)
        g_score = next((r["score"] for r in res if r["label"].lower() != "defect"), 0.0)
        is_def = d_score > g_score
        ms = int((datetime.utcnow() - t0).total_seconds() * 1000)
        return {"statusCode": 200, "headers": {"Content-Type": "application/json", "Access-Control-Allow-Origin": "*"}, "body": json.dumps({"request_id": rid, "detection": {"request_id": rid, "image_url": f"s3://{BUCKET}/{key}", "is_defective": is_def, "defect_probability": d_score, "confidence_score": max(d_score, g_score), "processing_time_ms": ms, "model": MODEL, "note": "Binary classification"}, "suggested_grade": "needs_inspection" if is_def else "likely_specialty"})}
    except Exception as e:
        return {"statusCode": 500, "headers": {"Content-Type": "application/json", "Access-Control-Allow-Origin": "*"}, "body": json.dumps({"error": str(e)})}
EOF`,
              'docker build -t $ECR_REPO_URI:latest .',
            ],
          },
          post_build: {
            commands: ['docker push $ECR_REPO_URI:latest'],
          },
        },
      }),
      timeout: cdk.Duration.minutes(30),
    });

    ecrRepo.grantPullPush(buildProject);

    // Use inline Python Lambda initially (will be replaced after CodeBuild)
    const aiLambda = new lambda.Function(this, 'AIDetectLambda', {
      functionName: 'coffee-qm-ai-detect',
      runtime: lambda.Runtime.PYTHON_3_11,
      handler: 'index.handler',
      code: lambda.Code.fromInline(`
import json, random
def handler(event, context):
    is_def = random.random() > 0.5
    return {"statusCode": 200, "headers": {"Content-Type": "application/json", "Access-Control-Allow-Origin": "*"}, "body": json.dumps({"request_id": "placeholder", "detection": {"is_defective": is_def, "defect_probability": 0.5, "confidence_score": 0.5, "processing_time_ms": 100, "model": "placeholder", "note": "Run CodeBuild to deploy real model"}, "suggested_grade": "needs_inspection" if is_def else "likely_specialty"})}
`),
      memorySize: 3008,
      timeout: cdk.Duration.seconds(60),
      environment: { BUCKET_NAME: imagesBucket.bucketName },
    });

    imagesBucket.grantReadWrite(aiLambda);

    // API Gateway
    const api = new apigateway.HttpApi(this, 'AIApi', {
      apiName: `coffee-qm-ai-${props.environment}`,
      corsPreflight: {
        allowOrigins: ['*'],
        allowMethods: [apigateway.CorsHttpMethod.POST, apigateway.CorsHttpMethod.OPTIONS],
        allowHeaders: ['*'],
      },
    });

    api.addRoutes({
      path: '/detect',
      methods: [apigateway.HttpMethod.POST],
      integration: new apigatewayIntegrations.HttpLambdaIntegration('AIIntegration', aiLambda),
    });

    // Outputs
    new cdk.CfnOutput(this, 'FrontendURL', { value: frontendBucket.bucketWebsiteUrl });
    new cdk.CfnOutput(this, 'AIApiURL', { value: api.apiEndpoint });
    new cdk.CfnOutput(this, 'FrontendBucketName', { value: frontendBucket.bucketName });
    new cdk.CfnOutput(this, 'CodeBuildProject', { value: buildProject.projectName });
    new cdk.CfnOutput(this, 'ECRRepository', { value: ecrRepo.repositoryUri });
    new cdk.CfnOutput(this, 'LambdaName', { value: aiLambda.functionName });
  }
}
