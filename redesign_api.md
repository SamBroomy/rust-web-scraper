# Restructure the api and layout the project a bit better

The reason I want to do this is to make the project more extensible and easier to work with. I want to make it easier to add new scrapers to the project.

## Changes

Current layout:

- src/
  - get_db/
    - mod.rs
  - main.rs
  - scraper/
    - error.rs
    - make_request.rs
    - mod.rs
    - pages/
      - bbc/
        - mod.rs
      - wikipedia/
        - mod.rs
    - url.rs

> Its a bit janky and not very clear what is what.

New layout:

- src/
  - get_db/
    - mod.rs
  - scraper/
    - mod.rs
    - common/
      - url.rs
      - make_request.rs
      - mod.rs
    - traits/
      - url.rs
      - scraper.rs
      - model.rs
      - mod.rs
    - sites/
      - mod.rs <- Scraper trait
      - bbc/
        - mod.rs
        - scraper.rs
        - model.rs
      - wikipedia/
        - mod.rs
        - scraper.rs
        - model.rs
      - another_site/
        - ...
  - main.rs

> If you have any suggestions or ideas, please let me know. Im still learning rust and how to design apis and projects in rust.

## Why

This way common basically will be traits and utils that are shared between all scrapers. Sites will be the scrapers for each site. This way we can easily add new scrapers to the project.

## Extras as part of the refractor

- Remove all the println statements, learn how to use logging and tracing.
- Handle errors better
- Add some tests
