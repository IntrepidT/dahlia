group "default" {
  targets = ["dahlia"]
}

target "dahlia" {
  context = "./dahlia"
  dockerfile = "./dahlia/Dockerfile"
  tags = ["intrepidt/dahlia:latest"]
  output = ["type=image"]
}
