#!/usr/bin/env node
import { spawnSync } from "child_process";
import { readFileSync, writeFileSync } from "fs";

const outputPath = process.argv[2] || ".beads/graph.html";

const { stdout, stderr, status } = spawnSync("bd", ["graph", "--all", "--html"], {
  encoding: "utf8",
  shell: true,
});

if (status !== 0) {
  console.error("bd graph --all --html failed:", stderr);
  process.exit(status);
}

const raw = stdout;
const doctypeCount = (raw.match(/<!DOCTYPE html>/g) || []).length;

if (doctypeCount <= 1) {
  writeFileSync(outputPath, raw);
  console.log(`Single document, wrote ${outputPath}`);
  process.exit(0);
}

console.log(`Found ${doctypeCount} connected components, combining into tabbed view`);

const parts = raw.split(/<!DOCTYPE html>/).filter(Boolean);
const tabs = [];

for (const part of parts) {
  const lines = part.split("\n");
  const titleLine = lines.find((l) => l.includes("<title>"));
  const title = titleLine
    ? titleLine.replace(/.*<title>(.*?)<\/title>.*/, "$1")
    : "Unknown";

  const scriptStart = lines.findIndex(
    (l) => l.includes("<script>") && !l.includes("src=")
  );
  const scriptEnd = lines.findLastIndex((l) => l.includes("</script>"));

  if (scriptStart === -1 || scriptEnd === -1) {
    console.warn(`Could not extract script from: ${title}`);
    continue;
  }

  const scriptBlock = lines.slice(scriptStart, scriptEnd + 1).join("\n");

  const nodesMatch = scriptBlock.match(
    /const nodes = (\[[\s\S]*?\]);/
  );
  const linksMatch = scriptBlock.match(
    /const links = (\[[\s\S]*?\]);|const links = (null);/
  );

  if (!nodesMatch || !linksMatch) {
    console.warn(`Could not extract nodes/links from: ${title}`);
    console.warn("  nodesMatch:", nodesMatch ? "found" : "missing");
    console.warn("  linksMatch:", linksMatch ? "found" : "missing");
    continue;
  }

  const titleId = title.replace(/[^a-zA-Z0-9]/g, "_");

  let linksData = nodesMatch ? null : null;
  if (linksMatch) {
    linksData = linksMatch[1] || linksMatch[2] || "[]";
    if (linksData === "null" || linksData === "null;") {
      linksData = "[]";
    }
  }

  tabs.push({
    title,
    titleId,
    nodes: nodesMatch[1],
    links: linksData || "[]",
  });
}

const combined = `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Beads: All Connected Components</title>
<style>
* { box-sizing: border-box; padding: 0; margin: 0; }
body { overflow: hidden; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif; color: #eee; background: #1a1a2e; }
svg { display: block; width: 100vw; height: 100vh; }
.node rect { rx: 6; ry: 6; stroke-width: 1.5; cursor: pointer; }
.node text { font-size: 11px; pointer-events: none; }
.node .id-text { font-size: 9px; fill: #999; }
.link { fill: none; stroke-width: 1.5; marker-end: url(#arrow); }
.link.blocks { stroke: #666; }
.link.parent-child { stroke: #555; stroke-dasharray: 5, 3; }
#tooltip { position: absolute; z-index: 10; max-width: 320px; padding: 10px 14px; font-size: 12px; pointer-events: none; background: #16213e; border: 1px solid #444; border-radius: 6px; opacity: 0; transition: opacity 0.15s; }
#tooltip .tt-id { font-weight: bold; color: #7ec8e3; }
#tooltip .tt-status { display: inline-block; padding: 1px 6px; margin-left: 6px; font-size: 10px; border-radius: 3px; }
#legend { position: absolute; top: 12px; right: 12px; padding: 12px 16px; font-size: 11px; background: rgba(22,33,62,0.9); border: 1px solid #333; border-radius: 8px; }
#legend h3 { margin-bottom: 6px; font-size: 12px; color: #7ec8e3; }
.legend-item { display: flex; gap: 6px; align-items: center; margin: 3px 0; }
.legend-swatch { display: inline-block; width: 14px; height: 14px; border-radius: 3px; }
#controls { position: absolute; bottom: 12px; left: 12px; padding: 8px 12px; font-size: 11px; background: rgba(22,33,62,0.9); border: 1px solid #333; border-radius: 8px; }
#controls button { padding: 4px 10px; margin: 0 2px; font-size: 11px; color: #ccc; cursor: pointer; background: #2a2a4a; border: 1px solid #444; border-radius: 4px; }
#controls button:hover { background: #3a3a5a; }
#tabs { position: absolute; top: 12px; left: 12px; display: flex; gap: 4px; z-index: 20; }
.tab-btn { padding: 6px 12px; font-size: 11px; color: #ccc; cursor: pointer; background: rgba(22,33,62,0.9); border: 1px solid #333; border-radius: 4px; }
.tab-btn.active { background: #4a9eff; border-color: #4a9eff; color: #fff; }
.tab-btn:hover:not(.active) { background: rgba(22,33,62,1); }
</style>
</head>
<body>
<div id="tabs">${tabs.map((t, i) => `<button class="tab-btn${i === 0 ? " active" : ""}" onclick="showTab(${i})">${t.title.replace(/^Beads: /, "")}</button>`).join("\n")}</div>
<div id="tooltip"></div>
<div id="legend">
  <h3>Status</h3>
  <div class="legend-item"><span class="legend-swatch" style="background:#4a9eff"></span> Open</div>
  <div class="legend-item"><span class="legend-swatch" style="background:#f0ad4e"></span> In Progress</div>
  <div class="legend-item"><span class="legend-swatch" style="background:#d9534f"></span> Blocked</div>
  <div class="legend-item"><span class="legend-swatch" style="background:#5cb85c"></span> Closed</div>
  <div class="legend-item"><span class="legend-swatch" style="background:#777"></span> Deferred</div>
  <h3 style="margin-top:8px">Edges</h3>
  <div class="legend-item"><svg width="30" height="10"><line x1="0" y1="5" x2="30" y2="5" stroke="#888" stroke-width="1.5"/></svg> blocks</div>
  <div class="legend-item"><svg width="30" height="10"><line x1="0" y1="5" x2="30" y2="5" stroke="#666" stroke-width="1.5" stroke-dasharray="5,3"/></svg> parent-child</div>
</div>
<div id="controls">
  <button onclick="resetZoom()">Reset View</button>
  <button onclick="toggleLabels()">Toggle Labels</button>
  Drag nodes to rearrange. Scroll to zoom.
</div>
<svg id="graph"></svg>
<script src="https://d3js.org/d3.v7.min.js"></script>
<script>
"use strict";
const statusColors = { open: "#4a9eff", in_progress: "#f0ad4e", blocked: "#d9534f", closed: "#5cb85c", deferred: "#777", hooked: "#9966cc" };
const width = window.innerWidth, height = window.innerHeight;
let showLabels = true;
let currentSim = null;

${tabs.map((t) => `
function render${t.titleId}() {
  const nodes = ${t.nodes};
  const links = ${t.links};
  const svg = d3.select("#graph");
  svg.selectAll("*").remove();
  if (currentSim) { currentSim.stop(); currentSim = null; }
  const defs = svg.append("defs");
  defs.append("marker").attr("id","arrow").attr("viewBox","0 -5 10 10").attr("refX",20).attr("refY",0).attr("markerWidth",6).attr("markerHeight",6).attr("orient","auto").append("path").attr("d","M0,-4L8,0L0,4").attr("fill","#666");
  const g = svg.append("g");
  const zoom = d3.zoom().scaleExtent([0.1,4]).on("zoom",(e)=>g.attr("transform",e.transform)); svg.call(zoom);
  currentSim = d3.forceSimulation(nodes).force("link",d3.forceLink(links).id(d=>d.id).distance(140).strength(0.7)).force("charge",d3.forceManyBody().strength(-400)).force("x",d3.forceX(d=>150+d.layer*220).strength(0.3)).force("y",d3.forceY(height/2).strength(0.05)).force("collision",d3.forceCollide(50));
  const link = g.append("g").selectAll("line").data(links).join("line").attr("class",d=>"link "+d.type).attr("stroke-dasharray",d=>d.type==="parent-child"?"5,3":null);
  const node = g.append("g").selectAll("g").data(nodes).join("g").attr("class","node").call(d3.drag().on("start",dragStart).on("drag",dragged).on("end",dragEnd));
  const nodeW=130,nodeH=40;
  node.append("rect").attr("width",nodeW).attr("height",nodeH).attr("x",-nodeW/2).attr("y",-nodeH/2).attr("fill",d=>statusColors[d.status]||"#555").attr("stroke",d=>d3.color(statusColors[d.status]||"#555").darker(0.5));
  node.append("text").attr("class","title-text").attr("text-anchor","middle").attr("dy",-3).text(d=>d.title.length>18?d.title.substring(0,17)+"\u2026":d.title);
  node.append("text").attr("class","id-text").attr("text-anchor","middle").attr("dy",12).text(d=>d.id+" P"+d.priority);
  const tooltip = d3.select("#tooltip");
  node.on("mouseover",(e,d)=>{tooltip.selectAll("*").remove();tooltip.text("");tooltip.append("span").attr("class","tt-id").text(d.id);tooltip.append("span").attr("class","tt-status").style("background",statusColors[d.status]||"#555").text(d.status);tooltip.append("br");tooltip.append("strong").text(d.title);tooltip.append("br");tooltip.append("span").text("Priority: P"+d.priority+" | Type: "+d.type);if(d.assignee){tooltip.append("br");tooltip.append("span").text("Assignee: "+d.assignee);}tooltip.append("br");tooltip.append("span").text("Layer: "+d.layer);tooltip.style("opacity",1).style("left",e.pageX+12+"px").style("top",e.pageY-10+"px");}).on("mouseout",()=>{tooltip.style("opacity",0);tooltip.selectAll("*").remove();});
  currentSim.on("tick",()=>{link.attr("x1",d=>d.source.x).attr("y1",d=>d.source.y).attr("x2",d=>d.target.x).attr("y2",d=>d.target.y);node.attr("transform",d=>"translate("+d.x+","+d.y+")");});
  svg.call(zoom.transform,d3.zoomIdentity.translate(width/4,height/4).scale(0.8));
}
`).join("")}

function showTab(idx) {
  document.querySelectorAll(".tab-btn").forEach((b,i)=>b.classList.toggle("active",i===idx));
  const fns = [${tabs.map((t) => "render" + t.titleId).join(",")}];
  if (fns[idx]) fns[idx]();
}
function resetZoom() { d3.select("#graph").transition().duration(500).call(d3.zoom().scaleExtent([0.1,4]).transform,d3.zoomIdentity.translate(width/4,height/4).scale(0.8)); }
function toggleLabels() { showLabels = !showLabels; d3.select("#graph").selectAll("text").style("opacity", showLabels ? 1 : 0); }
function dragStart(e,d){if(!e.active)currentSim.alphaTarget(0.3).restart();d.fx=d.x;d.fy=d.y;}
function dragged(e,d){d.fx=e.x;d.fy=e.y;}
function dragEnd(e,d){if(!e.active)currentSim.alphaTarget(0);d.fx=null;d.fy=null;}

showTab(0);
</script>
</body>
</html>`;

writeFileSync(outputPath, combined);
console.log(`Wrote ${outputPath} with ${tabs.length} tabs`);
