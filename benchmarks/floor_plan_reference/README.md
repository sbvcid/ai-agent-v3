# Floor Plan Reference Benchmark

This directory contains the deterministic reference geometry for the Single Room Residential Plan LLM implementation benchmark.

Files:

- `floor_plan_reference_v2.svg` — primary reference drawing, A3 landscape, vector SVG.
- `floor_plan.svg` — first generated reference version retained for provenance.
- `reference_spec.json` — structured fixed geometry and object specification.

The SVG is intended as the visual/geometry reference when comparing Kimi, Sonnet, DeepSeek, GPT and other LLM implementations. The benchmark should compare geometry and document quality rather than require identical SVG syntax.

Reference coordinate system:

- Building: 6000 x 5000 mm.
- Exterior walls: 200 mm.
- Interior partitions: 100 mm.
- Scale: 1:50.
- Sheet: A3 landscape.
- North: up.

For PDF generation on Windows, the simplest route is Inkscape CLI:

`inkscape floor_plan_reference_v2.svg --export-type=pdf --export-filename=floor_plan_reference.pdf`

The GitHub connector used for this session can write UTF-8 text files but cannot upload arbitrary binary PDF files directly. Therefore the SVG is committed as the authoritative vector reference; the PDF can be generated locally from that exact SVG without changing geometry.
