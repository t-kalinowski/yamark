args <- commandArgs(trailingOnly = TRUE)
stopifnot(length(args) == 1L, file.exists(args[[1L]]))
yamark <- normalizePath(args[[1L]], mustWork = TRUE)

stopifnot(
  requireNamespace("jsonlite", quietly = TRUE),
  requireNamespace("yaml12", quietly = TRUE)
)

flat_under_width <- c(
  '{"id":1,"name":"Ada","active":true}',
  '{"id":2,"name":"Grace","active":false}'
)
flat_over_width <- c(
  '{"id":1,"alpha":1,"beta":2,"gamma":3,"delta":4,"epsilon":5,"zeta":6,"eta":7,"theta":8}',
  '{"id":2,"alpha":1,"beta":2,"gamma":3,"delta":4,"epsilon":5,"zeta":6,"eta":7,"theta":8}'
)
nested_under_width <- c(
  '{"name":"Lin","active":true}',
  '{"name":"Sam","active":false}'
)
nested_over_width <- c(
  '{"alpha":1,"beta":2,"gamma":3,"delta":4,"epsilon":5,"zeta":6,"eta":7,"theta":8,"iota":9}',
  '{"alpha":1,"beta":2,"gamma":3,"delta":4,"epsilon":5,"zeta":6,"eta":7,"theta":8,"iota":9}'
)

stopifnot(
  all(nchar(flat_under_width, type = "width") < 80L),
  all(nchar(flat_over_width, type = "width") > 80L),
  all(nchar(nested_under_width, type = "width") < 80L),
  all(nchar(nested_over_width, type = "width") > 80L)
)

cases <- list(
  flat_under_width = flat_under_width,
  flat_over_width = flat_over_width,
  nested_under_width = paste0(
    '{"id":',
    seq_along(nested_under_width),
    ',"profile":',
    nested_under_width,
    "}"
  ),
  nested_over_width = paste0(
    '{"id":',
    seq_along(nested_over_width),
    ',"profile":',
    nested_over_width,
    "}"
  ),
  json_types = c(
    '{"id":1,"values":[1,2],"ratio":1.5,"missing":null,"enabled":true}',
    '{"id":2,"values":[3,4],"ratio":2.5,"missing":null,"enabled":false}',
    '{"id":3,"values":[5,6],"ratio":3.5,"missing":null,"enabled":true}'
  )
)

stopifnot(
  all(nchar(cases$nested_under_width, type = "width") < 80L),
  all(nchar(cases$nested_over_width, type = "width") > 80L)
)

read_json_lines <- function(path) {
  connection <- file(path, open = "rb")
  on.exit(close(connection), add = TRUE)
  jsonlite::stream_in(
    connection,
    simplifyVector = FALSE,
    verbose = FALSE
  )
}

run_yamark <- function(path) {
  command_output <- system2(
    yamark,
    c("format", "--line-width", "80", path),
    stdout = TRUE,
    stderr = TRUE
  )
  status <- attr(command_output, "status")
  if (is.null(status)) {
    status <- 0L
  }
  if (!identical(status, 0L)) {
    stop(paste(command_output, collapse = "\n"), call. = FALSE)
  }
}

root <- tempfile("yamark-json-lines-")
dir.create(root)
on.exit(unlink(root, recursive = TRUE), add = TRUE)

for (name in names(cases)) {
  path <- file.path(root, paste0(name, ".yaml"))
  input <- paste0(paste(cases[[name]], collapse = "\n"), "\n")
  writeBin(charToRaw(input), path)

  json_records <- read_json_lines(path)
  run_yamark(path)
  yaml_records <- yaml12::read_yaml(
    path,
    multi = TRUE,
    simplify = FALSE
  )

  if (!identical(json_records, yaml_records)) {
    stop(
      sprintf(
        paste0(
          "JSON and YAML records differ for %s\n",
          "JSON:\n%s\nYAML:\n%s"
        ),
        name,
        paste(capture.output(dput(json_records)), collapse = "\n"),
        paste(capture.output(dput(yaml_records)), collapse = "\n")
      ),
      call. = FALSE
    )
  }

  formatted <- readChar(path, file.info(path)$size, useBytes = TRUE)
  run_yamark(path)
  reformatted <- readChar(path, file.info(path)$size, useBytes = TRUE)
  if (!identical(formatted, reformatted)) {
    stop(sprintf("JSON Lines output is not idempotent for %s", name), call. = FALSE)
  }
}

cat(sprintf("JSON Lines semantic cases passed: %d\n", length(cases)))
