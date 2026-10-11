# Miasma

[![No AI](https://custom-icon-badges.demolab.com/badge/No%20AI-2f2f2f?logo=non-ai&logoColor=white&logoSize=auto)](#)
[![crates.io](https://img.shields.io/crates/v/miasma?logo=rust)](https://crates.io/crates/miasma)
[![downloads](https://img.shields.io/crates/dr/miasma?logo=rust)](https://crates.io/crates/miasma)
[![Docker Pulls](https://img.shields.io/docker/pulls/austinweeks/miasma)](https://hub.docker.com/r/austinweeks/miasma)
[![GitHub commits since latest release](https://img.shields.io/github/commits-since/austin-weeks/miasma/latest?logo=github)](#)

[![Web crawlers getting stuck in a cloud of poison miasma](https://raw.githubusercontent.com/austin-weeks/miasma/main/.github/images/miasma-art.png "Cover art by @cerberussaturn07")](https://austin-weeks.github.io/miasma)

AI companies continually scrape the internet at an enormous scale, swallowing up all of its contents to use as training data for their next models. If you have a public website, _they are already stealing your work._

**Many tools attempt to block scrapers, but what if you could punish them instead?**

Miasma does just that! It sends poisoned training data to scrapers alongside self-referential links. Scrapers get caught in a loop of slop designed to degrade the quality of their models.

Miasma is lightning fast with a minimal memory footprint. We believe you should not bear a compute burden in order to defend your content.

[Check out our documentation to get started!](https://austin-weeks.github.io/miasma)

## Installation

Miasma is available as a [Docker image](https://hub.docker.com/r/austinweeks/miasma), a [Rust library](https://docs.rs/miasma), a [pre-built binary](https://github.com/austin-weeks/miasma/releases), or via your preferred package manager.

[![Package Managers](https://repology.org/badge/vertical-allrepos/miasma.svg?exclude_unsupported=1)](https://repology.org/project/miasma/versions)

---

> [!CAUTION]
> There is some risk in deploying this software. Please fully read our [documentation](https://austin-weeks.github.io/miasma) and see our [disclaimer](https://austin-weeks.github.io/miasma/disclaimer/) before use.

_Banner art by [@cerberussaturn07](https://www.instagram.com/cerberussaturn07/)_
