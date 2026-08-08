(() => {
  "use strict";

  const SVG_NS = "http://www.w3.org/2000/svg";
  const hosts = "[data-benchmark-chart]";

  function svgNode(name, attributes = {}, text = null) {
    const node = document.createElementNS(SVG_NS, name);
    Object.entries(attributes).forEach(([key, value]) => {
      node.setAttribute(key, String(value));
    });
    if (text !== null) {
      node.textContent = text;
    }
    return node;
  }

  function chartSvg(width, height, id, title, description) {
    const svg = svgNode("svg", {
      viewBox: `0 0 ${width} ${height}`,
      preserveAspectRatio: "xMidYMid meet",
      class: "benchmark-chart-svg",
      "aria-labelledby": `${id}-title ${id}-description`,
    });
    svg.setAttribute("role", "img");
    svg.append(
      svgNode("title", { id: `${id}-title` }, title),
      svgNode("desc", { id: `${id}-description` }, description),
    );
    return svg;
  }

  function logDomain(values) {
    const logs = values.map((value) => Math.log10(value));
    const minimum = Math.min(...logs);
    const maximum = Math.max(...logs);
    const padding = Math.max((maximum - minimum) * 0.08, 0.08);
    return [minimum - padding, maximum + padding];
  }

  function logScale(domain, range) {
    const [domainStart, domainEnd] = domain;
    const [rangeStart, rangeEnd] = range;
    return (value) => {
      const position = (Math.log10(value) - domainStart) /
        (domainEnd - domainStart);
      return rangeStart + position * (rangeEnd - rangeStart);
    };
  }

  function tickValues(domain, compact) {
    const start = Math.floor(domain[0]);
    const end = Math.ceil(domain[1]);
    const multipliers = compact ? [1] : [1, 3];
    const ticks = [];
    for (let exponent = start; exponent <= end; exponent += 1) {
      multipliers.forEach((multiplier) => {
        const value = multiplier * (10 ** exponent);
        const log = Math.log10(value);
        if (log >= domain[0] && log <= domain[1]) {
          ticks.push(value);
        }
      });
    }
    return ticks;
  }

  function tickLabel(value) {
    if (value < 1) {
      return `${Math.round(value * 1000)} ms`;
    }
    return `${value.toLocaleString("en-US", { maximumFractionDigits: 0 })} s`;
  }

  function axis(svg, scale, ticks, top, bottom, left, right) {
    svg.append(svgNode("rect", {
      x: left,
      y: top,
      width: right - left,
      height: bottom - top,
      class: "benchmark-chart-frame",
    }));
    ticks.forEach((tick) => {
      const x = scale(tick);
      svg.append(
        svgNode("line", {
          x1: x,
          x2: x,
          y1: top,
          y2: bottom,
          class: "benchmark-chart-gridline",
        }),
        svgNode("text", {
          x,
          y: bottom + 20,
          class: "benchmark-chart-axis-label",
          "text-anchor": "middle",
        }, tickLabel(tick)),
      );
    });
  }

  function hideFallback(host) {
    const fallback = host.closest("figure")?.querySelector(
      ".benchmark-chart-fallback",
    );
    if (fallback) {
      fallback.hidden = true;
    }
  }

  function overviewDescription(rows) {
    return rows.map((row) => (
      `${row.workload}: Yamark ${row.yamark_duration}; ` +
      `${row.peer_formatter} ${row.peer_duration}. ${row.output_note}.`
    )).join(" ");
  }

  function renderOverview(host, rows, width) {
    const compact = width < 480;
    const rowHeight = compact ? 84 : 68;
    const top = compact ? 12 : 18;
    const bottomSpace = 42;
    const height = top + rows.length * rowHeight + bottomSpace;
    const plotLeft = compact ? 10 : Math.min(206, width * 0.31);
    const plotRight = width - (compact ? 10 : 18);
    const plotTop = top;
    const plotBottom = top + rows.length * rowHeight;
    const values = rows.flatMap((row) => [
      row.yamark_seconds,
      row.peer_seconds,
    ]);
    const domain = logDomain(values);
    const scale = logScale(domain, [plotLeft, plotRight]);
    const ticks = tickValues(domain, compact);
    const id = host.dataset.benchmarkSource.replace(/-data$/, "");
    const svg = chartSvg(
      width,
      height,
      `${id}-chart`,
      "Yamark and the next-lowest elapsed time",
      overviewDescription(rows),
    );

    axis(svg, scale, ticks, plotTop, plotBottom, plotLeft, plotRight);

    rows.forEach((row, index) => {
      const laneTop = top + index * rowHeight;
      const y = compact ? laneTop + 34 : laneTop + rowHeight / 2;
      const yamarkX = scale(row.yamark_seconds);
      const peerX = scale(row.peer_seconds);
      const peerAnchor = compact && peerX < plotLeft + 90 ? "start" : "end";

      if (compact) {
        svg.append(svgNode("text", {
          x: plotLeft,
          y: laneTop + 14,
          class: "benchmark-chart-workload benchmark-chart-workload-compact",
        }, row.short_workload));
      } else {
        svg.append(svgNode("text", {
          x: plotLeft - 14,
          y: y + 4,
          class: "benchmark-chart-workload",
          "text-anchor": "end",
        }, row.short_workload));
      }

      svg.append(
        svgNode("line", {
          x1: yamarkX,
          x2: peerX,
          y1: y,
          y2: y,
          class: "benchmark-dumbbell-line",
        }),
        svgNode("circle", {
          cx: yamarkX,
          cy: y,
          r: 5.5,
          class: "benchmark-mark benchmark-mark-yamark",
        }),
        svgNode("circle", {
          cx: peerX,
          cy: y,
          r: 5,
          class: "benchmark-mark benchmark-mark-peer benchmark-mark-peer-overview",
        }),
        svgNode("text", {
          x: yamarkX,
          y: y - 10,
          class: "benchmark-chart-value benchmark-chart-value-yamark",
          "text-anchor": "start",
        }, `Yamark · ${row.yamark_duration}`),
        svgNode("text", {
          x: peerX,
          y: y + 19,
          class: "benchmark-chart-value benchmark-chart-value-peer",
          "text-anchor": peerAnchor,
        }, `${row.peer_formatter} · ${row.peer_duration}`),
      );

      if (row.workload_id === "frontmatter") {
        svg.append(svgNode("text", {
          x: peerX,
          y: y + 33,
          class: "benchmark-chart-note",
          "text-anchor": peerAnchor,
        }, "front matter untouched"));
      }
    });

    svg.append(svgNode("text", {
      x: (plotLeft + plotRight) / 2,
      y: height - 4,
      class: "benchmark-chart-axis-title",
      "text-anchor": "middle",
    }, "Elapsed time (log scale)"));

    host.replaceChildren(svg);
    hideFallback(host);
  }

  function rowsByWorkload(rows) {
    const groups = new Map();
    rows.forEach((row) => {
      if (!groups.has(row.workload_order)) {
        groups.set(row.workload_order, []);
      }
      groups.get(row.workload_order).push(row);
    });
    return [...groups.values()].sort(
      (left, right) => left[0].workload_order - right[0].workload_order,
    );
  }

  function fullFieldDescription(rows) {
    return rows.map((row) => {
      let outcome = "";
      if (row.outcome) {
        outcome = row.workload_id === "frontmatter"
          ? `, front matter ${row.outcome}`
          : `, ${row.outcome}`;
      }
      return `${row.formatter} ${row.duration}${outcome}`;
    }).join("; ");
  }

  function renderFullFieldPanel(rows, domain, width, panelIndex) {
    const hasOutcomes = rows.some((row) => row.outcome);
    const rowHeight = hasOutcomes ? 42 : 35;
    const top = 12;
    const bottomSpace = 43;
    const height = top + rows.length * rowHeight + bottomSpace;
    const plotLeft = width < 430 ? 92 : 104;
    const plotRight = width - 12;
    const plotBottom = top + rows.length * rowHeight;
    const scale = logScale(domain, [plotLeft, plotRight]);
    const ticks = tickValues(domain, width < 520);
    const id = `benchmark-full-field-panel-${panelIndex + 1}`;
    const svg = chartSvg(
      width,
      height,
      id,
      `${rows[0].short_workload} elapsed time`,
      `${fullFieldDescription(rows)}. Lower elapsed time is better; ` +
        "the horizontal axis is logarithmic.",
    );

    axis(svg, scale, ticks, top, plotBottom, plotLeft, plotRight);

    rows.forEach((row, index) => {
      const y = top + index * rowHeight + rowHeight / 2;
      const x = scale(row.seconds);
      const atRight = x > plotLeft + (plotRight - plotLeft) * 0.76;

      svg.append(svgNode("text", {
        x: plotLeft - 10,
        y: y + (row.outcome ? -2 : 4),
        class: row.is_yamark
          ? "benchmark-chart-formatter benchmark-chart-formatter-yamark"
          : "benchmark-chart-formatter",
        "text-anchor": "end",
      }, row.formatter));

      if (row.outcome) {
        svg.append(svgNode("text", {
          x: plotLeft - 10,
          y: y + 12,
          class: "benchmark-chart-note",
          "text-anchor": "end",
        }, row.outcome));
      }

      svg.append(
        svgNode("circle", {
          cx: x,
          cy: y,
          r: row.is_yamark ? 5.5 : 4.5,
          class: row.is_yamark
            ? "benchmark-mark benchmark-mark-yamark"
            : "benchmark-mark benchmark-mark-peer",
        }),
        svgNode("text", {
          x: x + (atRight ? -8 : 8),
          y: y + 4,
          class: row.is_yamark
            ? "benchmark-chart-value benchmark-chart-value-yamark"
            : "benchmark-chart-value benchmark-chart-value-peer",
          "text-anchor": atRight ? "end" : "start",
        }, row.duration),
      );
    });

    svg.append(svgNode("text", {
      x: (plotLeft + plotRight) / 2,
      y: height - 4,
      class: "benchmark-chart-axis-title",
      "text-anchor": "middle",
    }, "Elapsed time (seconds, log scale)"));
    return svg;
  }

  function renderFullField(host, rows, width) {
    const groups = rowsByWorkload(rows);
    const values = rows.map((row) => row.seconds);
    const domain = logDomain(values);
    const columns = width >= 760 ? 2 : 1;
    const gap = 24;
    const panelWidth = columns === 2 ? (width - gap) / 2 : width;
    const grid = document.createElement("div");
    grid.className = "benchmark-full-field-grid";
    grid.style.gridTemplateColumns = columns === 2
      ? "repeat(2, minmax(0, 1fr))"
      : "minmax(0, 1fr)";

    groups.forEach((group, index) => {
      const panel = document.createElement("section");
      panel.className = "benchmark-full-field-panel";
      const heading = document.createElement("h4");
      heading.textContent = group[0].short_workload;
      panel.append(
        heading,
        renderFullFieldPanel(group, domain, panelWidth, index),
      );
      grid.append(panel);
    });

    host.replaceChildren(grid);
    hideFallback(host);
  }

  function initialize(host) {
    const source = document.getElementById(host.dataset.benchmarkSource);
    if (!source) {
      throw new Error(`Missing benchmark source: ${host.dataset.benchmarkSource}`);
    }
    const rows = JSON.parse(source.textContent);
    let frame = null;

    const render = () => {
      const width = Math.max(240, Math.floor(host.getBoundingClientRect().width));
      if (host.dataset.benchmarkChart === "overview") {
        renderOverview(host, rows, width);
      } else if (host.dataset.benchmarkChart === "full-field") {
        renderFullField(host, rows, width);
      } else {
        throw new Error(`Unknown benchmark chart: ${host.dataset.benchmarkChart}`);
      }
    };

    const observer = new ResizeObserver(() => {
      if (frame !== null) {
        cancelAnimationFrame(frame);
      }
      frame = requestAnimationFrame(render);
    });
    observer.observe(host);
    render();
  }

  function start() {
    document.querySelectorAll(hosts).forEach(initialize);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start, { once: true });
  } else {
    start();
  }
})();
