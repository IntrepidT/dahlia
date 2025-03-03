group "default" {
  targets = ["dahlia"]
}

target "dahlia" {
  context = "."
  dockerfile = "Dockerfile"
  tags = ["intrepidt/dahlia:latest"]
  output = ["type=image"]
}
