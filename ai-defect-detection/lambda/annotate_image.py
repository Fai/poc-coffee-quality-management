"""
Image Annotation Module for Coffee Bean Defect Detection

Generates annotated images with bounding boxes around detected defects,
color-coded by defect type.
"""

import io
import boto3
from PIL import Image, ImageDraw, ImageFont
from typing import Dict, List, Any, Tuple

# Initialize S3 client
s3 = boto3.client('s3')

# Color mapping for defect types (RGB)
DEFECT_COLORS = {
    # Category 1 (Primary) - Red shades
    'full_black': (255, 0, 0),        # Red
    'full_sour': (220, 20, 60),       # Crimson
    'pod_cherry': (178, 34, 34),      # Firebrick
    'large_stones': (139, 0, 0),      # Dark red
    'medium_stones': (165, 42, 42),   # Brown
    'large_sticks': (128, 0, 0),      # Maroon
    'medium_sticks': (160, 82, 45),   # Sienna
    # Category 2 (Secondary) - Orange/Yellow shades
    'partial_black': (255, 140, 0),   # Dark orange
    'partial_sour': (255, 165, 0),    # Orange
    'parchment': (255, 215, 0),       # Gold
    'floater': (255, 255, 0),         # Yellow
    'immature': (154, 205, 50),       # Yellow green
    'withered': (189, 183, 107),      # Dark khaki
    'shell': (240, 230, 140),         # Khaki
    'broken': (255, 127, 80),         # Coral
    'chipped': (255, 99, 71),         # Tomato
    'cut': (250, 128, 114),           # Salmon
    'insect_damage': (233, 150, 122), # Dark salmon
    'husk': (244, 164, 96),           # Sandy brown
    # Normal beans - Green
    'normal': (0, 128, 0),            # Green
}

# Thai translations for defect names
DEFECT_NAMES_TH = {
    'full_black': 'ดำเต็มเมล็ด',
    'full_sour': 'เปรี้ยวเต็มเมล็ด',
    'pod_cherry': 'เชอร์รี่ติดฝัก',
    'large_stones': 'หินใหญ่',
    'medium_stones': 'หินกลาง',
    'large_sticks': 'กิ่งไม้ใหญ่',
    'medium_sticks': 'กิ่งไม้กลาง',
    'partial_black': 'ดำบางส่วน',
    'partial_sour': 'เปรี้ยวบางส่วน',
    'parchment': 'กะลา',
    'floater': 'ลอย',
    'immature': 'ไม่สุก',
    'withered': 'เหี่ยว',
    'shell': 'เปลือก',
    'broken': 'แตก',
    'chipped': 'บิ่น',
    'cut': 'ตัด',
    'insect_damage': 'แมลงกัด',
    'husk': 'เปลือกแห้ง',
    'normal': 'ปกติ',
}


def annotate_image(
    image_bytes: bytes,
    detections: List[Dict[str, Any]],
    bucket_name: str,
    request_id: str
) -> str:
    """
    Annotate image with bounding boxes around detected defects.
    
    Args:
        image_bytes: Original image bytes
        detections: List of detection dictionaries with bbox, class_name, confidence
        bucket_name: S3 bucket name for storing annotated image
        request_id: Request ID for naming the output file
        
    Returns:
        S3 URL of the annotated image
    """
    # Load image
    image = Image.open(io.BytesIO(image_bytes))
    draw = ImageDraw.Draw(image)
    
    # Try to load a font, fall back to default if not available
    try:
        font = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf', 12)
        font_large = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf', 16)
    except:
        font = ImageFont.load_default()
        font_large = font
    
    # Draw bounding boxes for each detection
    for det in detections:
        bbox = det.get('bbox', [])
        class_name = det.get('class_name', 'unknown')
        confidence = det.get('confidence', 0)
        
        if len(bbox) != 4:
            continue
        
        x1, y1, x2, y2 = bbox
        color = DEFECT_COLORS.get(class_name, (128, 128, 128))
        
        # Draw bounding box
        draw.rectangle([x1, y1, x2, y2], outline=color, width=2)
        
        # Draw label background
        label = f'{class_name} {confidence:.0%}'
        label_bbox = draw.textbbox((x1, y1 - 15), label, font=font)
        draw.rectangle(label_bbox, fill=color)
        
        # Draw label text
        draw.text((x1, y1 - 15), label, fill=(255, 255, 255), font=font)
    
    # Add legend
    add_legend(image, detections)
    
    # Save annotated image to buffer
    output_buffer = io.BytesIO()
    image.save(output_buffer, format='JPEG', quality=90)
    output_buffer.seek(0)
    
    # Upload to S3
    annotated_key = f'annotated/{request_id}_annotated.jpg'
    s3.put_object(
        Bucket=bucket_name,
        Key=annotated_key,
        Body=output_buffer.getvalue(),
        ContentType='image/jpeg',
        Metadata={'request_id': request_id}
    )
    
    return f's3://{bucket_name}/{annotated_key}'


def add_legend(image: Image.Image, detections: List[Dict[str, Any]]) -> None:
    """
    Add a legend showing defect types and counts.
    
    Args:
        image: PIL Image to annotate
        detections: List of detections
    """
    # Count defects by type
    defect_counts = {}
    for det in detections:
        class_name = det.get('class_name', 'unknown')
        if class_name != 'normal':
            defect_counts[class_name] = defect_counts.get(class_name, 0) + 1
    
    if not defect_counts:
        return
    
    draw = ImageDraw.Draw(image)
    
    try:
        font = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf', 11)
    except:
        font = ImageFont.load_default()
    
    # Calculate legend position (bottom-right corner)
    img_width, img_height = image.size
    legend_width = 180
    legend_height = 20 + len(defect_counts) * 18
    legend_x = img_width - legend_width - 10
    legend_y = img_height - legend_height - 10
    
    # Draw legend background
    draw.rectangle(
        [legend_x, legend_y, legend_x + legend_width, legend_y + legend_height],
        fill=(255, 255, 255, 200),
        outline=(0, 0, 0)
    )
    
    # Draw legend title
    draw.text(
        (legend_x + 5, legend_y + 2),
        'Defects Found:',
        fill=(0, 0, 0),
        font=font
    )
    
    # Draw legend items
    y_offset = legend_y + 20
    for defect_type, count in sorted(defect_counts.items()):
        color = DEFECT_COLORS.get(defect_type, (128, 128, 128))
        
        # Color box
        draw.rectangle(
            [legend_x + 5, y_offset, legend_x + 15, y_offset + 10],
            fill=color,
            outline=(0, 0, 0)
        )
        
        # Defect name and count
        draw.text(
            (legend_x + 20, y_offset - 2),
            f'{defect_type}: {count}',
            fill=(0, 0, 0),
            font=font
        )
        
        y_offset += 18


def generate_summary_image(
    defect_counts: Dict[str, int],
    total_beans: int,
    grade: str,
    bucket_name: str,
    request_id: str
) -> str:
    """
    Generate a summary image showing defect statistics.
    
    Args:
        defect_counts: Dictionary of defect type to count
        total_beans: Total number of beans detected
        grade: Suggested grade classification
        bucket_name: S3 bucket name
        request_id: Request ID
        
    Returns:
        S3 URL of the summary image
    """
    # Create summary image
    width, height = 400, 500
    image = Image.new('RGB', (width, height), color=(255, 255, 255))
    draw = ImageDraw.Draw(image)
    
    try:
        font = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf', 14)
        font_large = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf', 18)
        font_title = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf', 20)
    except:
        font = ImageFont.load_default()
        font_large = font
        font_title = font
    
    # Title
    draw.text((20, 20), 'Coffee Bean Grading Summary', fill=(0, 0, 0), font=font_title)
    draw.text((20, 45), 'สรุปผลการเกรดเมล็ดกาแฟ', fill=(100, 100, 100), font=font)
    
    # Total beans
    draw.text((20, 80), f'Total Beans: {total_beans}', fill=(0, 0, 0), font=font_large)
    draw.text((20, 100), f'จำนวนเมล็ดทั้งหมด: {total_beans}', fill=(100, 100, 100), font=font)
    
    # Grade
    grade_display = grade.replace('_', ' ').title()
    draw.text((20, 130), f'Grade: {grade_display}', fill=(0, 0, 0), font=font_large)
    
    # Defect breakdown
    draw.text((20, 170), 'Defect Breakdown:', fill=(0, 0, 0), font=font_large)
    
    y_offset = 200
    
    # Category 1 defects
    cat1_total = sum(defect_counts.get(d, 0) for d in [
        'full_black', 'full_sour', 'pod_cherry', 
        'large_stones', 'medium_stones', 'large_sticks', 'medium_sticks'
    ])
    draw.text((20, y_offset), f'Category 1 (Primary): {cat1_total}', fill=(200, 0, 0), font=font)
    y_offset += 25
    
    # Category 2 defects
    cat2_total = sum(defect_counts.get(d, 0) for d in [
        'partial_black', 'partial_sour', 'parchment', 'floater',
        'immature', 'withered', 'shell', 'broken', 'chipped',
        'cut', 'insect_damage', 'husk'
    ])
    draw.text((20, y_offset), f'Category 2 (Secondary): {cat2_total}', fill=(255, 140, 0), font=font)
    y_offset += 25
    
    # Total defects
    total_defects = cat1_total + cat2_total
    draw.text((20, y_offset), f'Total Defects: {total_defects}', fill=(0, 0, 0), font=font_large)
    y_offset += 35
    
    # Individual defect counts (non-zero only)
    draw.text((20, y_offset), 'Details:', fill=(0, 0, 0), font=font)
    y_offset += 20
    
    for defect_type, count in sorted(defect_counts.items()):
        if count > 0:
            color = DEFECT_COLORS.get(defect_type, (128, 128, 128))
            thai_name = DEFECT_NAMES_TH.get(defect_type, defect_type)
            draw.rectangle([20, y_offset, 30, y_offset + 10], fill=color)
            draw.text((35, y_offset - 2), f'{defect_type} ({thai_name}): {count}', fill=(0, 0, 0), font=font)
            y_offset += 18
    
    # Save to buffer
    output_buffer = io.BytesIO()
    image.save(output_buffer, format='JPEG', quality=90)
    output_buffer.seek(0)
    
    # Upload to S3
    summary_key = f'summaries/{request_id}_summary.jpg'
    s3.put_object(
        Bucket=bucket_name,
        Key=summary_key,
        Body=output_buffer.getvalue(),
        ContentType='image/jpeg',
        Metadata={'request_id': request_id}
    )
    
    return f's3://{bucket_name}/{summary_key}'
