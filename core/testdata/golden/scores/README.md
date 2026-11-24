# Golden scores

OpenCV `TM_CCOEFF_NORMED` scores of templates searched in the frames under `core/testdata/synthetic/frames/`. Produced by `./gradlew :host:goldenScores`.

- Templates and masks are read from `core/testdata/synthetic/templates/`, not decoded from the game assets (OpenCV's gray decode differs from other PNG decoders). The template is `<template>`, the mask of a masked template is `<template without .png>.mask.png` (255 where the template pixel counts, 0 elsewhere).
- Only the region of each case is searched: the frame is cropped to `region` first, and `x`, `y` of `best` and `further` are the top-left of the match in frame coordinates (region origin added).
- The score map of a masked template has every value outside [-1.01, 1.01] (NaN included) replaced by 0 before anything else is done with it (`CvPattern.sanitizeMaskedResult`).
- `best` is the first maximum of the score map in row-major order as `Core.minMaxLoc` reports it, with no threshold.
- `further` comes from `CvPattern.findMatches` with similarity `similarity` (0.8): repeatedly take the maximum of the score map; stop when it is below 0.8; otherwise yield it and run `floodFill` on the score map from that location with fixed range, `loDiff = upDiff = 0.3` around the seed value, 4-connectivity, new value 0, so the connected pixels whose score is within 0.3 of the maximum become 0 (not a template-sized rectangle). The first yielded match is dropped (it is the best one when `best` reaches 0.8) and the next 3 at most are kept in the order found. A case whose best is below 0.8 has an empty `further`.
- Scores are float32 values printed as doubles (for example `1.0000001192092896`). The same OpenCV code gave `0.9999996423721313` on the machine that wrote the file and `1.0` on a GitHub Actions runner for the same case, so compare scores with a tolerance, never for equality.
