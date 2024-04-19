# Rust web scraper

This is a personal project to help me learn rust. The goal is to create a web scraper that can scrape wikipedia and or the bbc website. The scraper will be able to scrape the website and store the data in a document database (surreal db). The wikipedia data will be scraped where the document content will retain the nested structure of the wikipedia page. The bbc data will be scraped where the document content will be a flat structure.

The end goal is to embed the data into a knowledge graph and use the graph to answer questions.

There are a few things that I want to tick off while doing this project and they are shown below. I will update this list as I go along.

> Im relatively new to rust so I will be learning as I go along. Im sure there is probably a better way to implement some of the things I have done, if you see something that could be done better please let me know, any advice/help is welcome.

I am looking at other projects to see how they have implemented certain things where I can hopefully use some of the conventions and patterns they have used.

## Project Highlights

This is just a generic list of the things that I have used within the project, that I want to learn or have learnt (this dose not mean I have mastered sed thing it just means they have been used somewhere in the project, rightly or wrongly).

[x] Rust

  [x] Async
  [x] Structs
  [x] traits
  [x] Lifetimes
  [x] Error handling
  [-] Custom error types
  [ ] Macros
  [x] if-let
  [ ] let-else
  [ ] documentation
  [ ] tests

[x] Sql / NoSql

[x] Surreal DB / mainly for its hybrid document/graph model.

[x] Web scraping
  [ ] Depth first
  [x] Breadth first
  [ ] Handle categories

[ ] Good api and how it should be structured.
[x] Recursive Scraping
[x] Async
[x] Data Structures
[x] Algorithms

## Todo

Things that I would like to get to at some point.

- [ ] RAG Pipeline
- [ ] Web Server
- [ ] Tracing
- [ ] Python bindings
- [ ] Embeddings
- [ ] Knowledge Graph

Any feedback is welcome, I am always looking to improve.

Finally forgive the mess, this is simply a project to help me learn rust.
