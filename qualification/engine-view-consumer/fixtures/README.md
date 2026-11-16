# Retained mutation expectations

`engine-mutations.tsv` records offset, replacement byte and expected admission
for 189 mutations of the literal EngineView frame in the consumer test.
These data were captured from the previously passing paired corpus (Platform
0.33.7 and current Protocol), whose delivered source is
`60f9b9905eca010d4794603ef659d46bd3432d69`. They retain regression expectations,
not independent proof of Candid conformance. No legacy implementation is
copied or executed. Authored upstream Candid schemas and explicit literal
refusal classes provide separate independent coverage.
