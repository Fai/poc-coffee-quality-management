# AI Model Fine-Tuning Plan

## Current State

**Model**: `everycoffee/autotrain-coffee-bean-quality-97496146930`
- Type: Binary classification (defect/good)
- Limitation: No SCA defect type breakdown

**Target**: Full SCA defect detection with 18 defect types

---

## Phase 1: USK-Coffee Dataset Fine-Tuning (4 classes)

### Dataset
- **Source**: USK-Coffee Dataset (Universitas Syiah Kuala)
- **Size**: ~8,000 images
- **Classes**: 4 defect types (black, broken, faded, sour)
- **Format**: YOLO annotation format

### Model Selection
```
YOLOv8n (nano) - Recommended for Lambda deployment
- Size: ~6MB
- Inference: <100ms on CPU
- Accuracy: 97.7% precision (per research)
```

### Training Pipeline

```bash
# 1. Setup environment
pip install ultralytics torch torchvision

# 2. Download dataset
# Dataset structure:
# usk-coffee/
#   train/
#     images/
#     labels/
#   valid/
#     images/
#     labels/
#   data.yaml

# 3. Train
yolo detect train \
  model=yolov8n.pt \
  data=usk-coffee/data.yaml \
  epochs=100 \
  imgsz=640 \
  batch=16 \
  name=coffee-defect-v1

# 4. Export for Lambda
yolo export model=runs/detect/coffee-defect-v1/weights/best.pt format=onnx
```

### AWS Training (SageMaker)

```python
# sagemaker_training.py
import sagemaker
from sagemaker.pytorch import PyTorch

estimator = PyTorch(
    entry_point='train.py',
    source_dir='training',
    role='arn:aws:iam::ACCOUNT:role/SageMakerRole',
    instance_type='ml.g4dn.xlarge',  # GPU instance
    instance_count=1,
    framework_version='2.1',
    py_version='py310',
    hyperparameters={
        'epochs': 100,
        'batch-size': 16,
        'model': 'yolov8n'
    }
)

estimator.fit({'training': 's3://bucket/usk-coffee/'})
```

### Expected Results
| Metric | Target |
|--------|--------|
| mAP@50 | >95% |
| Precision | >97% |
| Recall | >95% |
| Inference | <100ms |

---

## Phase 2: Expand to Full SCA (18 classes)

### Additional Data Collection

| Category 1 (Primary) | Images Needed |
|---------------------|---------------|
| full_black | 500+ |
| full_sour | 500+ |
| pod_cherry | 300+ |
| large_stones | 200+ |
| medium_stones | 200+ |
| large_sticks | 200+ |
| medium_sticks | 200+ |

| Category 2 (Secondary) | Images Needed |
|-----------------------|---------------|
| partial_black | 500+ |
| partial_sour | 500+ |
| parchment | 300+ |
| floater | 300+ |
| immature | 500+ |
| withered | 300+ |
| shell | 300+ |
| broken | 500+ |
| chipped | 300+ |
| cut | 200+ |
| insect_damage | 300+ |
| husk | 200+ |

**Total**: ~6,000+ additional annotated images

### Data Collection Strategy

1. **Partner with Thai coffee farms** - Collect samples during grading
2. **Use existing grading sessions** - Photograph sorted defects
3. **Augmentation** - Rotate, flip, brightness variations
4. **Synthetic data** - Cut/paste defects onto clean backgrounds

### Annotation Tool
```
Label Studio (self-hosted) or Roboflow
- YOLO format export
- Team collaboration
- Quality control workflow
```

---

## Phase 3: Deployment Architecture

### Option A: Lambda + ONNX (Recommended for <1000 req/day)
```
Cost: ~$5/month
Latency: 2-5s cold start, <500ms warm
```

```python
# lambda_handler.py with ONNX
import onnxruntime as ort

session = ort.InferenceSession('model.onnx')

def predict(image):
    input_tensor = preprocess(image)
    outputs = session.run(None, {'images': input_tensor})
    return postprocess(outputs)
```

### Option B: SageMaker Serverless (1000-10000 req/day)
```
Cost: ~$30-50/month
Latency: 1-2s cold start, <200ms warm
```

### Option C: SageMaker Real-time (>10000 req/day)
```
Cost: ~$100+/month
Latency: <100ms consistent
```

---

## Timeline

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| Phase 1 | 2 weeks | 4-class YOLOv8 model |
| Phase 2 | 6-8 weeks | Data collection + annotation |
| Phase 2b | 2 weeks | 18-class model training |
| Phase 3 | 1 week | Production deployment |

---

## Cost Estimate

| Item | One-time | Monthly |
|------|----------|---------|
| SageMaker training (Phase 1) | $20 | - |
| Data annotation (Phase 2) | $500-1000 | - |
| SageMaker training (Phase 2) | $50 | - |
| Lambda inference | - | $5-10 |
| S3 storage | - | $5 |

**Total**: ~$600-1100 one-time + ~$10-15/month

---

## Next Steps

1. [ ] Download USK-Coffee dataset
2. [ ] Set up training environment (local or SageMaker)
3. [ ] Train YOLOv8n on 4 classes
4. [ ] Export to ONNX and test in Lambda
5. [ ] Begin Phase 2 data collection
