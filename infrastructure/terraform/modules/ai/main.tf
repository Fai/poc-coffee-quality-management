# AI Module - Lambda + API Gateway (pay-per-request)

variable "project" {}
variable "environment" {}

# S3 for images
resource "aws_s3_bucket" "ai_images" {
  bucket = "${lower(var.project)}-${var.environment}-ai-images-${data.aws_caller_identity.current.account_id}"
}

resource "aws_s3_bucket_lifecycle_configuration" "ai_images" {
  bucket = aws_s3_bucket.ai_images.id

  rule {
    id     = "delete-old-images"
    status = "Enabled"
    expiration { days = 7 }
  }
}

# Lambda IAM
resource "aws_iam_role" "lambda" {
  name = "${var.project}-${var.environment}-ai-lambda"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action    = "sts:AssumeRole"
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
    }]
  })
}

resource "aws_iam_role_policy" "lambda" {
  name = "ai-access"
  role = aws_iam_role.lambda.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect   = "Allow"
        Action   = ["logs:CreateLogGroup", "logs:CreateLogStream", "logs:PutLogEvents"]
        Resource = "arn:aws:logs:*:*:*"
      },
      {
        Effect   = "Allow"
        Action   = ["s3:PutObject", "s3:GetObject"]
        Resource = "${aws_s3_bucket.ai_images.arn}/*"
      },
      {
        Effect   = "Allow"
        Action   = "sagemaker:InvokeEndpoint"
        Resource = "*"
      }
    ]
  })
}

# Lambda Function
resource "aws_lambda_function" "defect_detection" {
  function_name = "${var.project}-${var.environment}-defect-detection"
  role          = aws_iam_role.lambda.arn
  handler       = "index.handler"
  runtime       = "python3.11"
  timeout       = 60
  memory_size   = 256

  filename         = data.archive_file.lambda.output_path
  source_code_hash = data.archive_file.lambda.output_base64sha256

  environment {
    variables = {
      BUCKET_NAME        = aws_s3_bucket.ai_images.id
      SAGEMAKER_ENDPOINT = "${var.project}-${var.environment}-defect"
    }
  }

  tags = { Component = "ai" }
}

data "archive_file" "lambda" {
  type        = "zip"
  output_path = "${path.module}/lambda.zip"

  source {
    content  = <<-EOF
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
          
          # Returns mock for testing - connect to SageMaker when model deployed
          return {
              'statusCode': 200,
              'headers': {'Content-Type': 'application/json'},
              'body': json.dumps({
                  'request_id': context.aws_request_id,
                  'detected_beans': 100,
                  'category1_count': 2,
                  'category2_count': 5,
                  'confidence': 0.92,
                  'suggested_grade': 'Specialty',
                  'note': 'Mock response - deploy SageMaker model for real inference'
              })
          }
    EOF
    filename = "index.py"
  }
}

# API Gateway
resource "aws_apigatewayv2_api" "ai" {
  name          = "${var.project}-${var.environment}-ai"
  protocol_type = "HTTP"
}

resource "aws_apigatewayv2_stage" "ai" {
  api_id      = aws_apigatewayv2_api.ai.id
  name        = var.environment
  auto_deploy = true
}

resource "aws_apigatewayv2_integration" "ai" {
  api_id                 = aws_apigatewayv2_api.ai.id
  integration_type       = "AWS_PROXY"
  integration_uri        = aws_lambda_function.defect_detection.invoke_arn
  payload_format_version = "2.0"
}

resource "aws_apigatewayv2_route" "ai" {
  api_id    = aws_apigatewayv2_api.ai.id
  route_key = "POST /detect"
  target    = "integrations/${aws_apigatewayv2_integration.ai.id}"
}

resource "aws_lambda_permission" "api" {
  statement_id  = "AllowAPIGateway"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.defect_detection.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.ai.execution_arn}/*/*"
}

data "aws_caller_identity" "current" {}

output "api_endpoint" {
  value = "${aws_apigatewayv2_api.ai.api_endpoint}/${var.environment}/detect"
}

output "images_bucket" {
  value = aws_s3_bucket.ai_images.id
}
