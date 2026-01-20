FROM public.ecr.aws/lambda/python:3.11

# Install dependencies
COPY requirements.txt ${LAMBDA_TASK_ROOT}/
RUN pip install --no-cache-dir -r ${LAMBDA_TASK_ROOT}/requirements.txt

# Download model at build time to avoid cold start download
RUN python -c "from transformers import pipeline; pipeline('image-classification', model='everycoffee/autotrain-coffee-bean-quality-97496146930')"

# Copy function code
COPY lambda_handler.py ${LAMBDA_TASK_ROOT}/

CMD ["lambda_handler.handler"]
