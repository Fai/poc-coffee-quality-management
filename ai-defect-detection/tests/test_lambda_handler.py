"""
Integration tests for AI Defect Detection Lambda Handler

Tests the complete flow from image upload to detection results.
"""

import json
import base64
import pytest
from unittest.mock import patch, MagicMock
import sys
import os

# Add lambda directory to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'lambda'))

from lambda_handler import (
    handler,
    calculate_grade,
    format_detection_result,
    generate_request_id,
    CATEGORY1_DEFECTS,
    CATEGORY2_DEFECTS
)


class TestGradeCalculation:
    """Tests for SCA grade classification."""
    
    def test_specialty_grade_zero_defects(self):
        """Specialty grade: 0 defects, 0 category 1."""
        assert calculate_grade(0, 0) == 'specialty_grade'
    
    def test_specialty_grade_max_defects(self):
        """Specialty grade: 5 defects, 0 category 1."""
        assert calculate_grade(0, 5) == 'specialty_grade'
    
    def test_specialty_disqualified_by_cat1(self):
        """Any category 1 defect disqualifies from specialty."""
        assert calculate_grade(1, 0) == 'premium_grade'
    
    def test_premium_grade(self):
        """Premium grade: 0-8 total defects."""
        assert calculate_grade(2, 4) == 'premium_grade'
        assert calculate_grade(0, 8) == 'premium_grade'
    
    def test_exchange_grade(self):
        """Exchange grade: 9-23 total defects."""
        assert calculate_grade(3, 6) == 'exchange_grade'
        assert calculate_grade(10, 13) == 'exchange_grade'
    
    def test_below_standard(self):
        """Below standard: 24-86 total defects."""
        assert calculate_grade(10, 14) == 'below_standard'
        assert calculate_grade(40, 46) == 'below_standard'
    
    def test_off_grade(self):
        """Off grade: 87+ total defects."""
        assert calculate_grade(40, 47) == 'off_grade'
        assert calculate_grade(100, 100) == 'off_grade'


class TestFormatDetectionResult:
    """Tests for detection result formatting."""
    
    def test_format_basic_result(self):
        """Test basic result formatting."""
        result = {
            'total_beans': 350,
            'defects': {
                'full_black': 1,
                'partial_black': 2,
                'broken': 3
            },
            'confidence': 0.95
        }
        
        formatted = format_detection_result(
            result,
            'det-test-123',
            'uploads/test.jpg',
            1500
        )
        
        assert formatted['request_id'] == 'det-test-123'
        assert formatted['detected_beans'] == 350
        assert formatted['category1_count'] == 1  # full_black
        assert formatted['category2_count'] == 5  # partial_black + broken
        assert formatted['confidence_score'] == 0.95
        assert formatted['processing_time_ms'] == 1500
    
    def test_format_empty_defects(self):
        """Test formatting with no defects."""
        result = {
            'total_beans': 350,
            'defects': {},
            'confidence': 0.98
        }
        
        formatted = format_detection_result(
            result,
            'det-test-456',
            'uploads/test.jpg',
            1000
        )
        
        assert formatted['category1_count'] == 0
        assert formatted['category2_count'] == 0
    
    def test_all_defect_types_included(self):
        """Test that all defect types are in the breakdown."""
        result = {
            'total_beans': 350,
            'defects': {},
            'confidence': 0.95
        }
        
        formatted = format_detection_result(
            result,
            'det-test-789',
            'uploads/test.jpg',
            1500
        )
        
        breakdown = formatted['defect_breakdown']
        
        # Check all category 1 defects
        for defect in CATEGORY1_DEFECTS:
            assert defect in breakdown
        
        # Check all category 2 defects
        for defect in CATEGORY2_DEFECTS:
            assert defect in breakdown


class TestRequestIdGeneration:
    """Tests for request ID generation."""
    
    def test_request_id_format(self):
        """Test request ID has correct format."""
        mock_context = MagicMock()
        mock_context.aws_request_id = 'test-aws-request-id'
        
        request_id = generate_request_id(mock_context)
        
        assert request_id.startswith('det-')
        assert len(request_id) > 20  # det- + timestamp + uuid


class TestLambdaHandler:
    """Integration tests for Lambda handler."""
    
    @patch('lambda_handler.s3')
    @patch('lambda_handler.endpoint_exists')
    def test_handler_success(self, mock_endpoint_exists, mock_s3):
        """Test successful detection request."""
        mock_endpoint_exists.return_value = False  # Use mock detection
        mock_s3.put_object.return_value = {}
        
        # Create a minimal test image (1x1 pixel JPEG)
        test_image = base64.b64encode(
            b'\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01\x01\x00\x00\x01\x00\x01\x00\x00'
            b'\xff\xdb\x00C\x00\x08\x06\x06\x07\x06\x05\x08\x07\x07\x07\t\t'
            b'\x08\n\x0c\x14\r\x0c\x0b\x0b\x0c\x19\x12\x13\x0f\x14\x1d\x1a'
            b'\x1f\x1e\x1d\x1a\x1c\x1c $.\' ",#\x1c\x1c(7),01444\x1f\'9teletext'
            b'\xff\xc0\x00\x0b\x08\x00\x01\x00\x01\x01\x01\x11\x00'
            b'\xff\xc4\x00\x1f\x00\x00\x01\x05\x01\x01\x01\x01\x01\x01\x00\x00'
            b'\x00\x00\x00\x00\x00\x00\x01\x02\x03\x04\x05\x06\x07\x08\t\n\x0b'
            b'\xff\xc4\x00\xb5\x10\x00\x02\x01\x03\x03\x02\x04\x03\x05\x05\x04'
            b'\x04\x00\x00\x01}\x01\x02\x03\x00\x04\x11\x05\x12!1A\x06\x13Qa'
            b'\x07"q\x142\x81\x91\xa1\x08#B\xb1\xc1\x15R\xd1\xf0$3br\x82\t\n'
            b'\x16\x17\x18\x19\x1a%&\'()*456789:CDEFGHIJSTUVWXYZcdefghijstuvwxyz'
            b'\xff\xda\x00\x08\x01\x01\x00\x00?\x00\xfb\xd5\x00\x00\x00\x00'
            b'\xff\xd9'
        ).decode('utf-8')
        
        event = {
            'body': json.dumps({
                'image_base64': test_image,
                'sample_weight_grams': 350.0
            })
        }
        
        context = MagicMock()
        context.aws_request_id = 'test-request-id'
        
        response = handler(event, context)
        
        assert response['statusCode'] == 200
        body = json.loads(response['body'])
        assert 'request_id' in body
        assert 'detection' in body
        assert 'suggested_grade' in body
    
    def test_handler_missing_image(self):
        """Test handler with missing image data."""
        event = {
            'body': json.dumps({
                'sample_weight_grams': 350.0
            })
        }
        
        context = MagicMock()
        
        response = handler(event, context)
        
        assert response['statusCode'] == 400
        body = json.loads(response['body'])
        assert 'error' in body
        assert 'image_base64' in body['error']
    
    def test_handler_invalid_base64(self):
        """Test handler with invalid base64 data."""
        event = {
            'body': json.dumps({
                'image_base64': 'not-valid-base64!!!'
            })
        }
        
        context = MagicMock()
        
        response = handler(event, context)
        
        assert response['statusCode'] == 400
        body = json.loads(response['body'])
        assert 'error' in body


class TestDefectCategories:
    """Tests for defect category definitions."""
    
    def test_category1_defects_defined(self):
        """Test all category 1 defects are defined."""
        expected = [
            'full_black', 'full_sour', 'pod_cherry',
            'large_stones', 'medium_stones', 'large_sticks', 'medium_sticks'
        ]
        assert set(CATEGORY1_DEFECTS) == set(expected)
    
    def test_category2_defects_defined(self):
        """Test all category 2 defects are defined."""
        expected = [
            'partial_black', 'partial_sour', 'parchment', 'floater',
            'immature', 'withered', 'shell', 'broken', 'chipped',
            'cut', 'insect_damage', 'husk'
        ]
        assert set(CATEGORY2_DEFECTS) == set(expected)
    
    def test_no_overlap_between_categories(self):
        """Test no defect appears in both categories."""
        overlap = set(CATEGORY1_DEFECTS) & set(CATEGORY2_DEFECTS)
        assert len(overlap) == 0


class TestGradeClassificationBoundaries:
    """Property-based tests for grade classification boundaries."""
    
    def test_specialty_boundary(self):
        """Test specialty grade boundary (5 defects, 0 cat1)."""
        assert calculate_grade(0, 5) == 'specialty_grade'
        assert calculate_grade(0, 6) == 'premium_grade'
    
    def test_premium_boundary(self):
        """Test premium grade boundary (8 defects)."""
        assert calculate_grade(4, 4) == 'premium_grade'
        assert calculate_grade(4, 5) == 'exchange_grade'
    
    def test_exchange_boundary(self):
        """Test exchange grade boundary (23 defects)."""
        assert calculate_grade(10, 13) == 'exchange_grade'
        assert calculate_grade(10, 14) == 'below_standard'
    
    def test_below_standard_boundary(self):
        """Test below standard boundary (86 defects)."""
        assert calculate_grade(43, 43) == 'below_standard'
        assert calculate_grade(43, 44) == 'off_grade'


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
