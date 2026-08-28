from qlty_check.src.utils import generate_random_id

def test_generate_random_id_happy_path():
    """Test that generate_random_id produces a string of the correct length"""
    result = generate_random_id(10)
    assert isinstance(result, str)
    assert len(result) == 10

def test_generate_random_id_edge_cases():
    """Test generate_random_id with edge case lengths"""
    # Test with length 0
    result = generate_random_id(0)
    assert isinstance(result, str)
    assert len(result) == 0
    
    # Test with larger length
    result = generate_random_id(100)
    assert isinstance(result, str)
    assert len(result) == 100

def test_generate_random_id_different_outputs():
    """Test that generate_random_id produces different outputs on subsequent calls"""
    result1 = generate_random_id(10)
    result2 = generate_random_id(10)
    assert result1 != result2
    
    # Ensure both are valid strings of same length
    assert isinstance(result1, str)
    assert isinstance(result2, str)
    assert len(result1) == 10
    assert len(result2) == 10