ZENSICAL_VERSION ?= 0.0.21

.PHONY: docs serve-docs

# Build documentation site
docs:
	@uvx --from "zensical==$(ZENSICAL_VERSION)" zensical build
	@touch site/.nojekyll

# Serve documentation locally with live reload
serve-docs:
	@uvx --from "zensical==$(ZENSICAL_VERSION)" zensical serve
