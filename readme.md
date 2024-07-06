# Rust web scraper

This is a personal project to help get more familiar with some of the deeper concepts in rust. The goal is to create an extensible web scraper that can scrape wikipedia and or the bbc website recursively. The scraper will be able to scrape the website and store the data in a document database (surreal db). The wikipedia data will be scraped where the document content will retain the nested structure of the wikipedia page. The bbc data will be scraped where the document content will be a flat structure.

The end goal is to embed the data into a knowledge graph and use the graph to answer questions.

There are a few things that I want to tick off while doing this project and they are shown below. I will update this list as I go along.

Im relatively new to rust so I will be learning as I go along. Im sure there is probably a better way to implement some of the things I have done, if you see something that could be done better please let me know, any advice/help is welcome. Some things I have implemented to simply learn how they work, so they may not be the best way to do it (eg PageState for example could be an enum but by having it as traits I can implement different types of page states for different types of states (Scrapable and Scraped that encompass multiple different page states.))

I am looking at other projects to see how they have implemented certain things where I can hopefully use some of the conventions and patterns they have used.

Any feedback is welcome, I am always looking to improve and learn.

Finally forgive the mess, this is simply another project to help me firm my grasp rust.

## Features

- Recursive web scraping
- Extensible architecture for adding new websites
- Asynchronous and concurrent scraping
- Data storage in SurrealDB (a multi-model database)
- Robust error handling and logging

## Project Highlights

This is just a generic list of the things that I have used within the project, that I want to learn or have learnt (this dose not mean I have mastered sed thing it just means they have been used somewhere in the project, rightly or wrongly).

- [x] Rust

  [x] Async

  [x] Smart Pointers

  [x] Concurrency

  [x] Structs

  [x] Generics

  [x] Traits

  [x] Closers

  [x] Dynamic Trait Objects

  [x] Lifetimes

  [x] Error handling

  [x] Custom error types

  [x] Macros

  [ ] documentation (rustdoc)

  [ ] tests

[x] Sql / NoSql

[x] Surreal DB / mainly for its hybrid document/graph model.

[x] Web scraping
  [x] Breadth first
  [x] Handle categories

[x] Recursive Scraping
[x] Async
[x] Data Structures
[x] Algorithms
[x] Tracing

## Prerequisites

Before running the project, ensure you have the following installed:

- Rust (latest stable version)
- [just](https://github.com/casey/just) - a command runner (install with `cargo install just`)
- [SurrealDB](https://surrealdb.com/) - for data storage

## Setup

1. Clone the repository:

2. Install SurrealDB if you haven't already. You can find installation instructions on the [SurrealDB website](https://surrealdb.com/install). *This is optional, you can use any database you like, but the project is configured to use SurrealDB by default.*

3. Install the `just` command runner:

   ```bash
   cargo install just
   ```

## Running the Project

1. Start the SurrealDB server:

   ```bash
   just db
   ```

   This command will start SurrealDB with the configuration specified in the `justfile`.

2. In a new terminal window, run the scraper:

   ```bash
   cargo run
   ```

The scraper will start with the BBC News homepage and recursively scrape linked articles, storing the data in SurrealDB.

## Project Structure

- `src/main.rs`: The entry point of the application
- `src/common/`: Contains common traits and structures used throughout the project
- `src/sites/`: Contains site-specific implementations (currently only BBC)
- `src/error.rs`: Defines custom error types for the project

## Extending the Scraper

To add support for a new website:

1. Create a new module in the `src/sites/` directory
2. Implement the necessary traits (`UrlTrait`, `ScrapableContent`, `SiteSpecificScraper`)
3. Update the main scraper to use the new site-specific scraper

## Extensibility

The project is designed to be extensible, so that you can define your own scrapable content and scrapers.

In the sites module you can define your own Urls, ScrapableContent and use these types with the page handler to scrape the content. Everything else is done for you.

!TODO macro to generate the boiler plate code for the scrapable content and the scraper.

## Todo

Things that I would like to get to at some point. Maybe not in this project but in a future project.

- [ ] RAG Pipeline
- [ ] Web Server
- [ ] Python bindings
- [ ] Embeddings
- [x] Knowledge Graph

## Notes

This project has been through two iterations, v1 & v2. The first iteration was a simply to get something working, it wasnt built very well and wasnt very extensible. The second iteration is much better and is designed to be extensible and modular. When changing the project from v1 to v2 it felt like at times I was fighting the borrow checker, but as I got more familiar with the concepts and the language it felt more like I was working with the borrow checker rather than against it (which is a nice feeling).  I have learnt a lot from this project and has definitely helped me get a deeper grasp of some of the concepts in rust.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Disclaimer

This project is for educational purposes only. Make sure to respect the terms of service of any website you scrape and be mindful of the load you put on their servers.
