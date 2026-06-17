#!/usr/bin/env Rscript

read_yaml_benchmark_results <- function(input_dir) {
  stopifnot(dir.exists(input_dir))
  stopifnot(requireNamespace("jsonlite", quietly = TRUE))

  files <- sort(list.files(input_dir, pattern = "[.]json$", full.names = TRUE))
  stopifnot(length(files) > 0L)

  rows <- lapply(files, function(path) {
    data <- jsonlite::fromJSON(path, simplifyVector = FALSE)
    stopifnot(identical(data$benchmark, "yaml-formatting"))

    ok_results <- Filter(function(result) identical(result$status, "ok"), data$results)
    lapply(ok_results, function(result) {
      data.frame(
        commit = data$git$short_commit,
        commit_time = data$git$commit_time,
        dirty = isTRUE(data$git$dirty),
        formatter = result$formatter,
        invocation = value_or(result$invocation, value_or(data$invocation$unit, "")),
        operation = value_or(result$operation, artifact_operation(data)),
        width_profile = value_or(
          result$width_profile,
          artifact_width_profile(data)
        ),
        corpus_shape = value_or(data$corpus$shape, ""),
        files = data$corpus$files,
        bytes = data$corpus$bytes,
        changed_files = result$changed_files,
        output_bytes = result$output_bytes,
        median_seconds = result$median_seconds,
        mean_seconds = result$mean_seconds,
        mb_per_second = result$mb_per_second,
        command = result$command,
        artifact = basename(path),
        stringsAsFactors = FALSE
      )
    })
  })

  rows <- do.call(c, rows)
  if (length(rows) == 0L) {
    return(data.frame(
      commit = character(),
      commit_time = character(),
      dirty = logical(),
      formatter = character(),
      invocation = character(),
      operation = character(),
      width_profile = character(),
      corpus_shape = character(),
      files = integer(),
      bytes = numeric(),
      changed_files = integer(),
      output_bytes = numeric(),
      median_seconds = numeric(),
      mean_seconds = numeric(),
      mb_per_second = numeric(),
      command = character(),
      artifact = character(),
      stringsAsFactors = FALSE
    ))
  }

  out <- do.call(rbind, rows)
  out[order(out$commit_time, out$formatter), ]
}

parse_args <- function(args) {
  out <- list(
    input_dir = file.path("docs", "benchmarks", "yaml"),
    formatter = "yamark",
    invocation = "per-file",
    operation = "write",
    width_profile = "default",
    limit_commits = 5L
  )
  index <- 1L
  while (index <= length(args)) {
    arg <- args[[index]]
    if (arg %in% c(
      "--input-dir", "--formatter", "--invocation", "--operation",
      "--width-profile", "--limit-commits"
    )) {
      index <- index + 1L
      stopifnot(index <= length(args))
      name <- sub("^--", "", arg)
      name <- gsub("-", "_", name)
      out[[name]] <- args[[index]]
    } else {
      stop("unknown argument: ", arg, call. = FALSE)
    }
    index <- index + 1L
  }
  out$limit_commits <- as.integer(out$limit_commits)
  stopifnot(out$limit_commits > 0L)
  out
}

main <- function() {
  args <- parse_args(commandArgs(trailingOnly = TRUE))
  results <- read_yaml_benchmark_results(args$input_dir)
  results <- filter_benchmark_results(
    results,
    formatter = args$formatter,
    invocation = args$invocation,
    operation = args$operation,
    width_profile = args$width_profile,
    limit_commits = args$limit_commits
  )
  stopifnot(requireNamespace("knitr", quietly = TRUE))

  table <- results[, c(
    "commit",
    "dirty",
    "formatter",
    "invocation",
    "operation",
    "width_profile",
    "corpus_shape",
    "files",
    "median_seconds",
    "mb_per_second"
  ), drop = FALSE]
  print(knitr::kable(table, format = "pipe", digits = 3, row.names = FALSE))
}

filter_benchmark_results <- function(
  results,
  formatter,
  invocation,
  operation,
  width_profile,
  limit_commits
) {
  out <- results
  if (nzchar(formatter)) {
    out <- out[identical_or_na(out$formatter, formatter), , drop = FALSE]
  }
  if (nzchar(invocation)) {
    out <- out[identical_or_na(out$invocation, invocation), , drop = FALSE]
  }
  if (nzchar(operation)) {
    out <- out[identical_or_na(out$operation, operation), , drop = FALSE]
  }
  if (nzchar(width_profile)) {
    out <- out[identical_or_na(out$width_profile, width_profile), , drop = FALSE]
  }
  if (nrow(out) == 0L) {
    return(out)
  }

  ordered <- out[order(out$commit_time, decreasing = TRUE), , drop = FALSE]
  commits <- unique(ordered$commit)
  keep <- commits[seq_len(min(limit_commits, length(commits)))]
  out <- out[out$commit %in% keep, , drop = FALSE]
  out[order(out$commit_time, out$formatter), , drop = FALSE]
}

identical_or_na <- function(values, expected) {
  !is.na(values) & values == expected
}

artifact_operation <- function(data) {
  if (is.character(data$operation)) data$operation else "write"
}

artifact_width_profile <- function(data) {
  value <- data$formatting_options$width_profile
  if (is.character(value)) value else "default"
}

value_or <- function(value, default) {
  if (is.null(value)) default else value
}

if (identical(environment(), globalenv()) && !length(sys.frames())) {
  main()
}
