# Shared benchmark-artifact readers and formatting helpers, sourced by
# index.qmd and benchmarks.qmd. Paths are relative to the website/ dir,
# which is the knitr working directory for both pages.

scalar <- function(x, default = NA) {
  value <- x %||% default
  if (length(value) == 0) {
    default
  } else {
    value[[1]]
  }
}

yaml_benchmark_dir <- normalizePath(
  file.path("..", "docs", "benchmarks", "yaml"),
  mustWork = TRUE
)
big_benchmark_dir <- normalizePath(
  file.path("..", "docs", "benchmarks", "big"),
  mustWork = TRUE
)
yaml_artifact_paths <- list.files(
  yaml_benchmark_dir,
  pattern = "\\.json$",
  full.names = TRUE
)
big_artifact_paths <- list.files(
  big_benchmark_dir,
  pattern = "\\.json$",
  full.names = TRUE
)
stopifnot(length(yaml_artifact_paths) > 0, length(big_artifact_paths) > 0)

artifact_version <- function(artifact, formatter) {
  as.character(scalar(artifact$tool_versions[[formatter]], NA))
}

read_yaml_artifact <- function(path) {
  artifact <- jsonlite::fromJSON(path, simplifyVector = FALSE)
  rows <- artifact$results %||% list()
  if (length(rows) == 0) {
    return(NULL)
  }
  short_commit <- scalar(
    artifact$git$short_commit,
    substr(scalar(artifact$git$commit, basename(path)), 1, 12)
  )
  do.call(
    rbind,
    lapply(rows, function(row) {
      formatter <- scalar(row$formatter, "")
      data.frame(
        artifact_path = path,
        artifact_file = basename(path),
        commit = short_commit,
        dirty = isTRUE(scalar(artifact$git$dirty, FALSE)),
        created_at = scalar(artifact$created_at, ""),
        host_system = scalar(artifact$host$system, ""),
        host_machine = scalar(artifact$host$machine, ""),
        host_cpu = as.character(scalar(artifact$host$cpu, NA)),
        corpus_shape = scalar(artifact$corpus$shape, ""),
        files = as.integer(scalar(artifact$corpus$files, NA)),
        items_per_file = as.integer(scalar(artifact$corpus$items_per_file, NA)),
        corpus_bytes = as.numeric(scalar(artifact$corpus$bytes, NA)),
        formatter = formatter,
        version = artifact_version(artifact, formatter),
        status = scalar(row$status, ""),
        invocation = scalar(row$invocation, ""),
        operation = scalar(row$operation, ""),
        reps = as.integer(scalar(row$reps, NA)),
        warmups = as.integer(scalar(row$warmups, NA)),
        median_seconds = as.numeric(scalar(row$median_seconds, NA)),
        mb_per_second = as.numeric(scalar(row$mb_per_second, NA)),
        median_user_seconds = as.numeric(scalar(row$median_user_seconds, NA)),
        stringsAsFactors = FALSE
      )
    })
  )
}

read_big_artifact <- function(path) {
  artifact <- jsonlite::fromJSON(path, simplifyVector = FALSE)
  stopifnot(identical(artifact$benchmark, "big-file-formatting"))
  rows <- artifact$results %||% list()
  if (length(rows) == 0) {
    return(NULL)
  }
  short_commit <- scalar(
    artifact$git$short_commit,
    substr(scalar(artifact$git$commit, basename(path)), 1, 12)
  )
  requested <- artifact$corpus$requested_bytes
  do.call(
    rbind,
    lapply(rows, function(row) {
      formatter <- scalar(row$formatter, "")
      data.frame(
        artifact_path = path,
        artifact_file = basename(path),
        commit = short_commit,
        created_at = scalar(artifact$created_at, ""),
        host_system = scalar(artifact$host$system, ""),
        host_machine = scalar(artifact$host$machine, ""),
        host_cpu = as.character(scalar(artifact$host$cpu, NA)),
        requested_markdown = as.numeric(scalar(requested$markdown, NA)),
        requested_yaml = as.numeric(scalar(requested$yaml, NA)),
        requested_frontmatter = as.numeric(scalar(requested$frontmatter, NA)),
        requested_frontmatter_yaml = as.numeric(
          scalar(requested$frontmatter_yaml, NA)
        ),
        target_file = scalar(row$file, ""),
        target_bytes = as.numeric(scalar(row$bytes, NA)),
        formatter = formatter,
        version = artifact_version(artifact, formatter),
        status = scalar(row$status, ""),
        reason = as.character(scalar(row$reason, NA)),
        reps = as.integer(scalar(row$reps, NA)),
        warmups = as.integer(scalar(row$warmups, NA)),
        median_seconds = as.numeric(scalar(row$median_seconds, NA)),
        median_user_seconds = as.numeric(scalar(row$median_user_seconds, NA)),
        median_peak_rss_bytes = as.numeric(scalar(row$median_peak_rss_bytes, NA)),
        front_matter = as.character(scalar(row$front_matter, NA)),
        stringsAsFactors = FALSE
      )
    })
  )
}

benchmark_rows <- do.call(
  rbind,
  Filter(Negate(is.null), lapply(yaml_artifact_paths, read_yaml_artifact))
)
big_rows_all <- do.call(
  rbind,
  Filter(Negate(is.null), lapply(big_artifact_paths, read_big_artifact))
)
stopifnot(nrow(benchmark_rows) > 0, nrow(big_rows_all) > 0)

# One comparison roster per input kind: every native formatter CLI for that
# input, used simply (no formatting options, no shims, no adapters). Lint
# fixers (pymarkdown, markdownlint-cli2) are not formatters and library
# read/write baselines (py-yaml12, pretty-yaml, panache-yaml) are not CLIs a
# user would format with, so neither group is displayed.
markdown_formatters <- c(
  "yamark",
  "panache",
  "mdformat",
  "prettier",
  "dprint-markdown",
  "deno-fmt"
)
yaml_formatters <- c(
  "yamark",
  "yamlfmt",
  "prettier",
  "yamlfix",
  "dprint-yaml",
  "deno-fmt"
)

markdown_target <- "big.md"
yaml_target <- "big.yaml"
frontmatter_target <- "big-with-frontmatter.md"
big_targets <- c(markdown_target, yaml_target, frontmatter_target)

target_roster <- function(target_file) {
  switch(
    target_file,
    "big.md" = markdown_formatters,
    "big.yaml" = yaml_formatters,
    "big-with-frontmatter.md" = markdown_formatters,
    stop("unknown big-file target: ", target_file)
  )
}

# Corpus sizes the page describes; an artifact generated at other sizes is
# never silently rendered. 4 MB keeps every comparison tool in play
# (panache refuses inputs larger than 4 MiB), and the front matter block is
# 5% of the document's bytes, which is about a third of its lines.
big_requested_bytes <- 4000000
big_requested_frontmatter_yaml <- 200000
directory_files <- 500
directory_items <- 540

parse_created_at <- function(x) {
  as.POSIXct(x, format = "%Y-%m-%dT%H:%M:%OSZ", tz = "UTC")
}

latest_artifact <- function(keep) {
  stopifnot(length(keep) > 0)
  keep[[which.max(parse_created_at(
    vapply(keep, function(rows) rows$created_at[[1]], character(1))
  ))]]
}

# Latest artifact in which every comparison formatter completed the requested
# invocation with status "ok". A degraded run (missing or failed tool) is
# never silently rendered as a smaller table.
select_yaml_artifact <- function(shape, files, items_per_file, invocation) {
  candidates <- split(benchmark_rows, benchmark_rows$artifact_path)
  keep <- Filter(function(rows) {
    ok <- rows[rows$invocation == invocation & rows$status == "ok", , drop = FALSE]
    identical(rows$corpus_shape[[1]], shape) &&
      identical(rows$files[[1]], as.integer(files)) &&
      identical(rows$items_per_file[[1]], as.integer(items_per_file)) &&
      all(yaml_formatters %in% ok$formatter)
  }, candidates)
  latest_artifact(keep)
}

# Latest artifact in which every roster tool completed its target with
# status "ok". A degraded run (missing, skipped, or failed tool) is never
# silently rendered as a smaller table.
select_big_artifact <- function() {
  candidates <- split(big_rows_all, big_rows_all$artifact_path)
  keep <- Filter(function(rows) {
    ok <- rows[rows$status == "ok", , drop = FALSE]
    isTRUE(rows$requested_markdown[[1]] == big_requested_bytes) &&
      isTRUE(rows$requested_yaml[[1]] == big_requested_bytes) &&
      isTRUE(rows$requested_frontmatter[[1]] == big_requested_bytes) &&
      isTRUE(
        rows$requested_frontmatter_yaml[[1]] == big_requested_frontmatter_yaml
      ) &&
      all(vapply(
        big_targets,
        function(target) {
          all(target_roster(target) %in% ok$formatter[ok$target_file == target])
        },
        logical(1)
      ))
  }, candidates)
  latest_artifact(keep)
}

flow_rows <- select_yaml_artifact(
  "flow-heavy",
  directory_files,
  directory_items,
  "directory"
)
flow_directory_rows <- flow_rows[
  flow_rows$invocation == "directory" &
    flow_rows$status == "ok" &
    flow_rows$formatter %in% yaml_formatters,
]
flow_directory_rows <- flow_directory_rows[
  order(flow_directory_rows$median_seconds),
]
# The page says yamark is the fastest on wall time and on single-core user
# CPU; fail the render rather than publish either sentence against an
# artifact where it is false.
stopifnot(
  identical(flow_directory_rows$formatter[[1]], "yamark"),
  which.min(flow_directory_rows$median_user_seconds) ==
    which(flow_directory_rows$formatter == "yamark")
)

big_rows <- select_big_artifact()
big_ok_rows <- big_rows[big_rows$status == "ok", ]

big_target_rows <- function(target_file) {
  roster <- target_roster(target_file)
  rows <- big_ok_rows[
    big_ok_rows$target_file == target_file &
      big_ok_rows$formatter %in% roster, ,
    drop = FALSE
  ]
  stopifnot(all(roster %in% rows$formatter))
  rows <- rows[order(rows$median_seconds), ]
  # Fail the render rather than claim yamark is fastest where it is not.
  stopifnot(identical(rows$formatter[[1]], "yamark"))
  rows
}

# The front-matter table reports who rewrote the deliberately unformatted
# front matter block; yamark must be among them or the corpus is broken.
stopifnot(identical(
  big_ok_rows$front_matter[
    big_ok_rows$formatter == "yamark" &
      big_ok_rows$target_file == frontmatter_target
  ],
  "rewritten"
))

fmt_sec <- function(x) sprintf("%.3f s", x)
fmt_duration <- function(x) {
  ifelse(x < 1, sprintf("%.0f ms", x * 1000), sprintf("%.1f s", x))
}
fmt_mbps <- function(x) {
  ifelse(x >= 10, sprintf("%.1f MB/s", x), sprintf("%.2f MB/s", x))
}
fmt_mb <- function(bytes) sprintf("%.1f MB", bytes / 1e6)
fmt_mb_round <- function(bytes) sprintf("%.0f MB", bytes / 1e6)
fmt_kb <- function(bytes) sprintf("%.0f KB", bytes / 1e3)
artifact_url <- function(rows) {
  artifact_kind <- basename(dirname(rows$artifact_path[[1]]))
  paste0(
    "https://github.com/t-kalinowski/yamark/blob/main/docs/benchmarks/",
    artifact_kind,
    "/",
    rows$artifact_file[[1]]
  )
}
artifact_link <- function(rows) {
  sprintf("[`%s`](%s)", substr(rows$commit[[1]], 1, 7), artifact_url(rows))
}
slower_than <- function(seconds, base) {
  ratio <- seconds / base
  if (abs(ratio - 1) < 0.005) {
    "1x"
  } else if (ratio < 1) {
    sprintf("%.2fx faster", 1 / ratio)
  } else {
    sprintf("%.1fx slower", ratio)
  }
}
row_value <- function(rows, formatter, column) {
  rows[rows$formatter == formatter, column][[1]]
}

directory_yamark_seconds <- row_value(
  flow_directory_rows, "yamark", "median_seconds"
)
directory_next_row <- flow_directory_rows[
  flow_directory_rows$formatter != "yamark", ,
  drop = FALSE
][1, , drop = FALSE]
stopifnot(nrow(directory_next_row) == 1)

big_table <- function(target_file, front_matter = FALSE) {
  rows <- big_target_rows(target_file)
  base <- rows$median_seconds[rows$formatter == "yamark"][[1]]
  out <- data.frame(
    Formatter = rows$formatter,
    Time = fmt_duration(rows$median_seconds),
    Memory = fmt_mb(rows$median_peak_rss_bytes),
    check.names = FALSE
  )
  if (front_matter) {
    out[["Front matter"]] <- vapply(
      rows$front_matter,
      function(outcome) {
        switch(
          outcome,
          rewritten = "formatted",
          preserved = "untouched",
          removed = "not preserved",
          stop("unknown front matter outcome: ", outcome)
        )
      },
      character(1)
    )
  }
  out[["vs yamark"]] <- vapply(
    rows$median_seconds,
    slower_than,
    character(1),
    base = base
  )
  out
}

big_seconds <- function(target_file, formatter) {
  row_value(big_target_rows(target_file), formatter, "median_seconds")
}

big_bytes <- function(target_file) {
  big_target_rows(target_file)$target_bytes[[1]]
}

big_next_row <- function(target_file) {
  rows <- big_target_rows(target_file)
  rows[rows$formatter != "yamark", , drop = FALSE][1, , drop = FALSE]
}

# One-sentence headlines shared by the home page and the benchmarks page.
bigfile_headline <- function() {
  markdown_next <- big_next_row(markdown_target)
  yaml_next <- big_next_row(yaml_target)
  sprintf(
    "**Yamark formats a %s Markdown document in %s and a %s YAML file in %s.** The next-fastest tool on each is `%s` (%s) and `%s` (%s).",
    fmt_mb_round(big_bytes(markdown_target)),
    fmt_duration(big_seconds(markdown_target, "yamark")),
    fmt_mb_round(big_bytes(yaml_target)),
    fmt_duration(big_seconds(yaml_target, "yamark")),
    markdown_next$formatter[[1]],
    fmt_duration(markdown_next$median_seconds[[1]]),
    yaml_next$formatter[[1]],
    fmt_duration(yaml_next$median_seconds[[1]])
  )
}

directory_headline <- function() {
  sprintf(
    "On a directory of %d YAML files (%s), yamark finishes in %s; the next-fastest formatter, `%s`, takes %s.",
    flow_directory_rows$files[[1]],
    fmt_mb_round(flow_directory_rows$corpus_bytes[[1]]),
    fmt_duration(directory_yamark_seconds),
    directory_next_row$formatter[[1]],
    fmt_duration(directory_next_row$median_seconds[[1]])
  )
}

# Inline comma-separated roster of every tool yamark is compared against.
compared_tools_inline <- function() {
  tools <- sort(unique(c(markdown_formatters, yaml_formatters)))
  tools <- setdiff(tools, "yamark")
  paste0("`", tools, "`", collapse = ", ")
}

# "median of N runs after M warmups", taken from the artifact so the prose
# can never drift from how the published numbers were actually measured.
measurement_inline <- function(rows = flow_directory_rows) {
  reps <- rows$reps[[1]]
  warmups <- rows$warmups[[1]]
  stopifnot(!is.na(reps), reps >= 2, !is.na(warmups), warmups >= 1)
  sprintf(
    "the median of %d measured runs after %d warmup run%s",
    reps,
    warmups,
    if (warmups == 1) "" else "s"
  )
}

host_inline <- function() {
  cpu <- unique(c(flow_directory_rows$host_cpu, big_ok_rows$host_cpu))
  system <- unique(c(flow_directory_rows$host_system, big_ok_rows$host_system))
  machine <- unique(c(
    flow_directory_rows$host_machine,
    big_ok_rows$host_machine
  ))
  stopifnot(
    length(cpu) == 1, !is.na(cpu), nzchar(cpu),
    length(system) == 1, nzchar(system),
    length(machine) == 1, nzchar(machine)
  )
  label <- if (identical(system, "Darwin")) "macOS" else system
  sprintf("%s, %s %s", cpu, label, machine)
}

# Inline tool-version list across both artifacts. Both tables must come from
# the same tool versions; a version that differs between artifacts fails the
# render instead of publishing one of the two numbers.
tool_versions_block <- function() {
  big_roster_rows <- big_rows[
    big_rows$formatter %in% c(markdown_formatters, yaml_formatters), ,
    drop = FALSE
  ]
  rows <- rbind(
    flow_directory_rows[, c("formatter", "version")],
    big_roster_rows[, c("formatter", "version")]
  )
  rows <- rows[!is.na(rows$version), , drop = FALSE]
  conflicting <- vapply(
    split(rows$version, rows$formatter),
    function(versions) length(unique(versions)) > 1,
    logical(1)
  )
  stopifnot(!any(conflicting))
  rows <- rows[!duplicated(rows$formatter), , drop = FALSE]
  if (nrow(rows) == 0) {
    return("")
  }
  version <- sub(": ", " ", rows$version, fixed = TRUE)
  version <- ifelse(
    grepl("[A-Za-z]", version),
    version,
    paste(rows$formatter, version)
  )
  version <- unique(version)
  paste0(
    "Tool versions: ",
    paste0("`", version, "`", collapse = ", "),
    "."
  )
}

write_table <- function(data, align = NULL) {
  cat(knitr::kable(
    data,
    format = "html",
    escape = FALSE,
    align = align,
    row.names = FALSE,
    table.attr = 'class="perf-table"'
  ))
}
