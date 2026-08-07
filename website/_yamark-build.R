yamark_bin <- Sys.getenv("YAMARK_BIN", unset = "")
if (nzchar(yamark_bin)) {
  yamark_bin <- normalizePath(yamark_bin, winslash = "/", mustWork = TRUE)
} else {
  status <- system2(
    "cargo",
    c(
      "build",
      "--release",
      "--bin",
      "yamark",
      "--manifest-path",
      "../Cargo.toml"
    )
  )
  if (!identical(status, 0L)) {
    stop("`cargo build` failed with status ", status, call. = FALSE)
  }

  yamark_exe <- if (.Platform$OS.type == "windows") "yamark.exe" else "yamark"
  yamark_bin <- normalizePath(
    file.path("..", "target", "release", yamark_exe),
    winslash = "/",
    mustWork = TRUE
  )
}

trim_one_trailing_newline <- function(x) {
  sub("\n\\z", "", x, perl = TRUE)
}

yamark_format <- function(input, stdin_file_path, args = character()) {
  stopifnot(is.character(input), length(input) == 1)
  stopifnot(is.character(stdin_file_path), length(stdin_file_path) == 1)
  stopifnot(is.character(args))

  stderr_path <- tempfile()
  on.exit(unlink(stderr_path), add = TRUE)

  output <- system2(
    yamark_bin,
    args = c(
      "format",
      "--wrap",
      "72",
      "--stdin-file-path",
      stdin_file_path,
      args
    ),
    input = strsplit(input, "\n", fixed = TRUE)[[1]],
    stdout = TRUE,
    stderr = stderr_path
  )

  status <- attr(output, "status")
  if (!is.null(status) && !identical(status, 0L)) {
    stderr <- paste(readLines(stderr_path, warn = FALSE), collapse = "\n")
    stop(
      "`",
      yamark_bin,
      " format` failed with status ",
      status,
      " for ",
      stdin_file_path,
      if (nzchar(stderr)) paste0(":\n", stderr) else "",
      call. = FALSE
    )
  }

  trim_one_trailing_newline(paste(output, collapse = "\n"))
}

markdown_fence <- function(x, lang = "markdown") {
  runs <- regmatches(x, gregexpr("`+", x, perl = TRUE))[[1]]
  longest_run <- if (length(runs) == 0) 0 else max(nchar(runs))
  fence <- strrep("`", max(3, longest_run + 1))

  paste0(fence, lang, "\n", x, "\n", fence)
}
