variable "region" {
  type = map(any)
  default = {
    "a" = {
      "region" = "uswest",
    }
  }
}

variable "foo" {
  default = "bar"
}
