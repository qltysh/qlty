provider "aws" {
  access_key = "hardcoded-access-key"
  secret_key = "hardcoded-secret-key"
  region     = "us-east-1"
}

resource "aws_s3_bucket" "insecure_bucket" {
  bucket = "my-insecure-bucket"

  tags = {
    Name        = "MyInsecureBucket"
    Environment = "Production"
  }
}

resource "aws_security_group" "bad_sg" {
  name        = "allow-all-sg"
  description = "Security group that allows all traffic"

  ingress {
    from_port   = 0
    to_port     = 65535
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
}
