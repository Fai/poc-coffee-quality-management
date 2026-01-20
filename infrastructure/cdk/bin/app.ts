#!/usr/bin/env node
import * as cdk from 'aws-cdk-lib';
import { CoffeeQMStack } from '../lib/coffee-qm-stack';
import { CoffeeAIStack } from '../lib/coffee-ai-stack';

const app = new cdk.App();

const env = { account: process.env.CDK_DEFAULT_ACCOUNT, region: 'ap-southeast-1' };

// AI Stack (Frontend + Lambda with HuggingFace model)
new CoffeeAIStack(app, 'CoffeeQM-AI', {
  env,
  environment: 'test',
});

// Full stack with backend (uncomment when needed)
// new CoffeeQMStack(app, 'CoffeeQM-test', {
//   env,
//   environment: 'test',
//   costCenter: 'CC-COFFEE-001',
//   monthlyBudget: 50,
//   useFargateSpot: true,
//   dbInstanceClass: 'db.t3.micro',
// });

app.synth();
