# Rust web scraper

This is a personal project to help get more familiar with some of the deeper concepts in rust. The goal is to create an extensible web scraper that can scrape wikipedia and or the bbc website recursively. The scraper will be able to scrape the website and store the data in a document database (surreal db). The wikipedia data will be scraped where the document content will retain the nested structure of the wikipedia page. The bbc data will be scraped where the document content will be a flat structure.

The end goal is to embed the data into a knowledge graph and use the graph to answer questions.

There are a few things that I want to tick off while doing this project and they are shown below. I will update this list as I go along.

> Im relatively new to rust so I will be learning as I go along. Im sure there is probably a better way to implement some of the things I have done, if you see something that could be done better please let me know, any advice/help is welcome. Some things I have implemented to simply learn how they work, so they may not be the best way to do it (eg PageState for example could be an enum but by having it as traits I can implement different types of page states for different types of states (Scrapable and Scraped that encompass multiple different page states.))

I am looking at other projects to see how they have implemented certain things where I can hopefully use some of the conventions and patterns they have used.

Any feedback is welcome, I am always looking to improve and learn.

Finally forgive the mess, this is simply another project to help me firm my grasp rust.

# Project Highlights

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

  [-] Lifetimes

  [x] Error handling

  [-] Custom error types

  [x] Macros

  [ ] documentation (rustdoc)

  [ ] tests

[x] Sql / NoSql

[x] Surreal DB / mainly for its hybrid document/graph model.

[x] Web scraping
  [x] Breadth first
  [x] Handle categories

[-] Good api and how it should be structured.
[x] Recursive Scraping
[x] Async
[x] Data Structures
[x] Algorithms
[ ] Tracing

## Concepts

The project is designed to be extensible and modular. The [core concepts](./src/scraper_v2/common/) are:

### Urls (trait)

What should url that you are trying to scrape look like.

### PageState (trait)

The state of the page, if it has been scraped or not.

### Page (struct)

Page is a struct that represents a page (a Url and a page state). Its url is the associated url of the page and the state represent essentially if the content of the page has been scraped yet or not.

### ScrapableContent (Trait)

This is defining what exactly it is you want to scrape from the page (e.g. the title, the content, the links etc).

> Due to the extensibility, you can define many different types of scrapable content for the same page. aka given a page (yet to be scraped) you can scrape it for different types of content.

(TODO! - have a Fetched state which will fetch the HTML content of the page, and thus allow for several different content types to be created/scraped without making more than one request).

### PageScraper (Trait)

Defines the sub scraper, for a given url type and content type.

### PageHandler (struct)

The scraper is the component that orchestrates the recursive scraping of the pages.

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
- [ ] Knowledge Graph

## Notes

This project has been through two iterations, v1 & v2. The first iteration was a simply to get something working, it wasnt built very well and wasnt very extensible. The second iteration is much better and is designed to be extensible and modular. When changing the project from v1 to v2 it felt like at times I was fighting the borrow checker, but as I got more familiar with the concepts and the language it felt more like I was working with the borrow checker rather than against it (which is a nice feeling).  I have learnt a lot from this project and has definitely helped me get a deeper grasp of some of the concepts in rust.
