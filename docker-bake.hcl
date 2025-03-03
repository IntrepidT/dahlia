group "default" {
  targets = ["dahlia"]
}

target "dahlia" {
  context = "."
  dockerfile = "./dahlia/Dockerfile"
  tags = ["intrepidt/dahlia:latest"]
  output = ["type=image"]
}
