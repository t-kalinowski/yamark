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
        changed_files = as.integer(scalar(row$changed_files, NA)),
        output_files = as.integer(scalar(row$output_files, NA)),
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
        changed = as.logical(scalar(row$changed, NA)),
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
# The page says yamark has the lowest elapsed time; fail the render rather
# than publish that sentence against an artifact where it is false.
stopifnot(identical(flow_directory_rows$formatter[[1]], "yamark"))

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
fmt_duration_range <- function(x) {
  if (all(x < 1)) {
    sprintf("%.0f–%.0f ms", min(x) * 1000, max(x) * 1000)
  } else {
    sprintf("%s–%s", fmt_duration(min(x)), fmt_duration(max(x)))
  }
}
fmt_mb <- function(bytes) sprintf("%.1f MB", bytes / 1e6)
fmt_mb_round <- function(bytes) sprintf("%.0f MB", bytes / 1e6)
fmt_kb <- function(bytes) sprintf("%.0f KB", bytes / 1e3)

formatter_label <- function(formatter) {
  switch(
    formatter,
    yamark = "Yamark",
    panache = "Panache",
    mdformat = "mdformat",
    prettier = "Prettier",
    `dprint-markdown` = "dprint",
    `dprint-yaml` = "dprint",
    `deno-fmt` = "Deno",
    yamlfmt = "yamlfmt",
    yamlfix = "yamlfix",
    stop("unknown formatter: ", formatter)
  )
}

front_matter_label <- function(outcome) {
  switch(
    outcome,
    rewritten = "formatted",
    preserved = "untouched",
    removed = "not preserved",
    stop("unknown front matter outcome: ", outcome)
  )
}

artifact_url <- function(rows) {
  artifact_kind <- basename(dirname(rows$artifact_path[[1]]))
  paste0(
    "https://github.com/t-kalinowski/yamark/blob/main/docs/benchmarks/",
    artifact_kind,
    "/",
    rows$artifact_file[[1]]
  )
}
artifact_link <- function(rows, label) {
  sprintf("[%s](%s)", label, artifact_url(rows))
}
row_value <- function(rows, formatter, column) {
  rows[rows$formatter == formatter, column][[1]]
}

directory_yamark_seconds <- row_value(
  flow_directory_rows, "yamark", "median_seconds"
)

big_table <- function(target_file, front_matter = FALSE) {
  rows <- big_target_rows(target_file)
  out <- data.frame(
    Formatter = rows$formatter,
    `Wall time` = fmt_duration(rows$median_seconds),
    `Peak RSS` = fmt_mb(rows$median_peak_rss_bytes),
    check.names = FALSE
  )
  if (front_matter) {
    out[["Front matter"]] <- vapply(
      rows$front_matter,
      front_matter_label,
      character(1)
    )
  }
  out
}

big_seconds <- function(target_file, formatter) {
  row_value(big_target_rows(target_file), formatter, "median_seconds")
}

big_bytes <- function(target_file) {
  big_target_rows(target_file)$target_bytes[[1]]
}

benchmark_workload_rows <- function(
  id,
  label,
  short_label,
  order,
  rows
) {
  stopifnot(
    nrow(rows) >= 2,
    sum(rows$formatter == "yamark") == 1,
    all(is.finite(rows$median_seconds)),
    all(rows$median_seconds > 0)
  )
  rank <- rank(rows$median_seconds, ties.method = "first")
  changed <- if ("changed" %in% names(rows)) {
    rows$changed
  } else {
    rows$changed_files == rows$files
  }
  stopifnot(!anyNA(changed))
  outcome <- rep(NA_character_, nrow(rows))
  if ("changed" %in% names(rows)) {
    outcome[!is.na(rows$changed) & !rows$changed] <- "file unchanged"
  }
  if (identical(id, "frontmatter")) {
    outcome <- vapply(rows$front_matter, front_matter_label, character(1))
  }
  data.frame(
    workload_id = id,
    workload = label,
    short_workload = short_label,
    workload_order = as.integer(order),
    formatter_id = rows$formatter,
    formatter = vapply(rows$formatter, formatter_label, character(1)),
    seconds = rows$median_seconds,
    duration = fmt_duration(rows$median_seconds),
    rank = as.integer(rank),
    is_yamark = rows$formatter == "yamark",
    changed = changed,
    outcome = outcome,
    stringsAsFactors = FALSE
  )
}

# Canonical long-form elapsed-time data for every benchmark presentation.
# The overview and summary table are derived from these rows so the three
# views cannot select different formatters or report different values.
benchmark_full_field_rows <- function() {
  markdown_rows <- big_target_rows(markdown_target)
  yaml_rows <- big_target_rows(yaml_target)
  frontmatter_rows <- big_target_rows(frontmatter_target)

  stopifnot(
    all(flow_directory_rows$changed_files == flow_directory_rows$files),
    all(flow_directory_rows$output_files == flow_directory_rows$files)
  )

  rows <- rbind(
    benchmark_workload_rows(
      "markdown",
      "4 MB Markdown",
      "4 MB Markdown",
      1,
      markdown_rows
    ),
    benchmark_workload_rows(
      "yaml",
      "4 MB YAML",
      "4 MB YAML",
      2,
      yaml_rows
    ),
    benchmark_workload_rows(
      "frontmatter",
      "4 MB Markdown + 200 KB YAML front matter",
      "4 MB Markdown + front matter",
      3,
      frontmatter_rows
    ),
    benchmark_workload_rows(
      "directory",
      "500 YAML files (50 MB)",
      "500 YAML files · 50 MB",
      4,
      flow_directory_rows
    )
  )
  rows <- rows[order(rows$workload_order, rows$rank), , drop = FALSE]
  rownames(rows) <- NULL
  rows
}

benchmark_summary_rows <- function() {
  rows <- benchmark_full_field_rows()
  by_workload <- split(rows, rows$workload_order)
  do.call(rbind, lapply(by_workload, function(workload_rows) {
    yamark <- workload_rows[workload_rows$is_yamark, , drop = FALSE]
    peers <- workload_rows[!workload_rows$is_yamark, , drop = FALSE]
    peer <- peers[which.min(peers$seconds), , drop = FALSE]
    stopifnot(nrow(yamark) == 1, nrow(peer) == 1)
    stopifnot(isTRUE(yamark$changed[[1]]))
    output_note <- switch(
      yamark$workload_id[[1]],
      markdown = if (isTRUE(peer$changed[[1]])) {
        "Both rewrite Markdown"
      } else {
        sprintf("%s leaves the generated Markdown unchanged", peer$formatter)
      },
      yaml = if (isTRUE(peer$changed[[1]])) {
        "Both rewrite YAML"
      } else {
        sprintf("%s leaves the generated YAML unchanged", peer$formatter)
      },
      frontmatter = switch(
        peer$outcome[[1]],
        formatted = "Both format YAML front matter",
        untouched = sprintf(
          "%s leaves YAML front matter untouched",
          peer$formatter
        ),
        `not preserved` = sprintf(
          "%s does not preserve YAML front matter",
          peer$formatter
        ),
        stop("unknown front matter outcome: ", peer$outcome[[1]])
      ),
      directory = {
        stopifnot(isTRUE(peer$changed[[1]]))
        sprintf("Both rewrite all %d files", flow_directory_rows$files[[1]])
      },
      stop("unknown workload: ", yamark$workload_id[[1]])
    )
    data.frame(
      workload_id = yamark$workload_id,
      workload = yamark$workload,
      short_workload = yamark$short_workload,
      workload_order = yamark$workload_order,
      yamark_seconds = yamark$seconds,
      yamark_duration = yamark$duration,
      peer_formatter = peer$formatter,
      peer_seconds = peer$seconds,
      peer_duration = peer$duration,
      peer_per_yamark = peer$seconds / yamark$seconds,
      output_note = output_note,
      stringsAsFactors = FALSE
    )
  }))
}

benchmark_summary_table <- function() {
  rows <- benchmark_summary_rows()
  data.frame(
    Workload = rows$workload,
    Yamark = rows$yamark_duration,
    `Next-lowest elapsed` = paste(rows$peer_formatter, rows$peer_duration, sep = " · "),
    `Peer / Yamark` = sprintf("%.1f×", rows$peer_per_yamark),
    `Output note` = rows$output_note,
    check.names = FALSE
  )
}

write_benchmark_chart <- function(kind, id, title, subtitle, rows) {
  stopifnot(
    kind %in% c("overview", "full-field"),
    grepl("^[a-z][a-z0-9-]+$", id),
    nrow(rows) > 0
  )
  seconds <- if (identical(kind, "overview")) {
    c(rows$yamark_seconds, rows$peer_seconds)
  } else {
    rows$seconds
  }
  stopifnot(all(is.finite(seconds)), all(seconds > 0))

  source_id <- paste0(id, "-data")
  chart_columns <- if (identical(kind, "overview")) {
    c(
      "workload_id", "workload", "short_workload",
      "yamark_seconds", "yamark_duration",
      "peer_formatter", "peer_seconds", "peer_duration", "output_note"
    )
  } else {
    c(
      "workload_id", "short_workload", "workload_order",
      "formatter", "seconds", "duration", "is_yamark", "outcome"
    )
  }
  chart_rows <- rows[, chart_columns, drop = FALSE]
  json <- jsonlite::toJSON(
    chart_rows,
    dataframe = "rows",
    auto_unbox = TRUE,
    na = "null",
    digits = 15
  )
  json <- gsub("</", "<\\/", json, fixed = TRUE)
  escape <- htmltools::htmlEscape
  fallback <- if (identical(kind, "overview")) {
    items <- vapply(seq_len(nrow(rows)), function(index) {
      row <- rows[index, , drop = FALSE]
      sprintf(
        paste0(
          "<li><strong>%s:</strong> Yamark %s; %s %s. %s.</li>"
        ),
        escape(row$workload),
        escape(row$yamark_duration),
        escape(row$peer_formatter),
        escape(row$peer_duration),
        escape(row$output_note)
      )
    }, character(1))
    paste0(
      '<div class="benchmark-chart-fallback"><p>Exact values:</p><ul>',
      paste(items, collapse = ""),
      "</ul></div>"
    )
  } else {
    paste0(
      '<p class="benchmark-chart-fallback">',
      "Exact values are available in the detailed benchmark tables below.</p>"
    )
  }

  cat(sprintf(
    paste0(
      '<figure class="benchmark-chart benchmark-%s-chart" ',
      'aria-labelledby="%s-caption">\n',
      '<figcaption id="%s-caption">\n',
      '<h3>%s</h3>\n',
      '<p>%s</p>\n',
      '</figcaption>\n',
      '<div class="benchmark-chart-canvas" data-benchmark-chart="%s" ',
      'data-benchmark-source="%s"></div>\n',
      '%s\n',
      '<script type="application/json" id="%s">%s</script>\n',
      '</figure>\n'
    ),
    escape(kind),
    escape(id),
    escape(id),
    escape(title),
    escape(subtitle),
    escape(kind),
    escape(source_id),
    fallback,
    escape(source_id),
    json
  ))
}

# One-sentence headlines shared by the home page and the benchmarks page.
benchmark_headline <- function() {
  big_yamark_seconds <- vapply(
    big_targets,
    function(target) big_seconds(target, "yamark"),
    numeric(1)
  )
  big_sizes <- unique(vapply(
    big_targets,
    function(target) fmt_mb_round(big_bytes(target)),
    character(1)
  ))
  stopifnot(length(big_sizes) == 1)
  sprintf(
    "**On the benchmark host (%s), Yamark recorded the lowest elapsed time in all four workloads:** %s for each generated %s file and %s for %d generated YAML files (%s).",
    host_inline(),
    fmt_duration_range(big_yamark_seconds),
    big_sizes,
    fmt_duration(directory_yamark_seconds),
    flow_directory_rows$files[[1]],
    fmt_mb_round(flow_directory_rows$corpus_bytes[[1]])
  )
}

# Inline comma-separated roster of every tool Yamark is compared against.
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

write_table <- function(
  data,
  align = NULL,
  table_class = "perf-table",
  caption = NULL
) {
  cat(knitr::kable(
    data,
    format = "html",
    escape = FALSE,
    align = align,
    row.names = FALSE,
    caption = caption,
    table.attr = sprintf('class="%s"', table_class)
  ))
}
