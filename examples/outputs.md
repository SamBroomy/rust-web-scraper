# Data scraped from the BBC News website

## Pages

Below is an example of the data scraped from a BBC News article. The data is in JSON format and contains the following fields:

- `content`: The content of the article.
- `id`: The ID of the article.
- `title`: The title of the article.
- `metadata`: Metadata about the article, such as the timestamp, related topics, and page links.
  - `page_links`: Links to other pages related to the article.
  - `related_topics`: Topics related to the article.
  - `timestamp`: The timestamp of the article.
  - `url`: The URL of the article. (Same as the `id` field)

### Sample from `pages` table

```json
// pages:⟨/news/world-us-canada-63978323⟩
{
 content: [
  "Elon Musk says he is taking legal action against the holder of a Twitter account that tracks his private jet, arguing it put his son at risk.The @ElonJet account, external, which has more than half a million followers, was suspended on Wednesday.Its owner Jack Sweeney, 20, used publicly available flight-tracking information to tweet every time Mr Musk's jet took off and landed. Mr Musk says legal action is now being taken against Mr Sweeney and others.\"Last night, car carrying [his son] lil X in LA was followed by crazy stalker (thinking it was me), who later blocked car from moving and climbed onto hood,\" he tweeted.He added that any account revealing people's real-time locations will be suspended \"as it is a physical safety violation\".",
  "Elon Musk sells $3.6bn of Tesla sharesTwitter Files spark debate about ‘blacklisting’Elon Musk no longer world's richest man",
  "Mr Sweeney denied the incident was related to his account when asked by the BBC.It comes after he confirmed, external on his personal Twitter account on Wednesday that the profile had been suspended. That evening, Mr Sweeney's account appeared to have been reactivated. He tweeted: \"Yes I am back!\" Minutes later it was listed again as suspended. His personal account, @JxckSweeney, has also been frozen.Mr Sweeney, a college student in the state of Florida, shared a screenshot with CNN, external of a message from Twitter saying the social media company had conducted a \"careful review\" and had decided to permanently ban the account for violating Twitter's rules.The student is in charge of dozens of other accounts that track the private flights of wealthy Americans, including Microsoft co-founder Bill Gates, Amazon founder Jeff Bezos, and Meta Chief Executive Officer Mark Zuckerberg. Many of those accounts - including one tracking aircraft associated with Russian President Vladimir Putin, and another monitoring celebrity jets - appeared to be suspended on Twitter as well on Wednesday afternoon.  Mr Musk had long taken issue with the @ElonJet account, and once reportedly offered Mr Sweeney $5,000 to delete it. Mr Sweeney told US media outlets that Mr Musk ultimately told him it did not feel right to pay to have the account shut down. And a month ago, Mr Musk pledged to keep it running even though it was a \"direct personal safety risk\". But Mr Musk tweeted on Wednesday evening: \"Any account doxxing real-time location info of anyone will be suspended, as it is a physical safety violation. This includes posting links to sites with real-time location info.\"Twitter's Help Center has tweeted an updated media policy, external that begins: \"You may not publish or post other people's private information without their express authorization and permission.\" Since taking the helm at Twitter, Mr Musk has made a host of changes to its moderation practices. He has restored a handful of previously banned accounts, including former President Donald Trump's profile, which was banned following the 6 January insurrection at the US Capitol. ",
  "The Tesla CEO has also slashed the social media company's staff and has reportedly stopped paying rent for some of Twitter's offices, including the company's San Francisco headquarters, according to the New York Times, external. Investors have questioned whether his recent takeover of Twitter has diverted his attention from his electric car business.On Monday, Tuesday and Wednesday of this week, he sold another 22 million shares, worth $3.58bn (£2.9bn), in the company.It brings the total of Tesla stocks sold by Mr Musk over the past year to almost $40bn."
 ],
 id: pages:⟨/news/world-us-canada-63978323⟩,
 metadata: {
  page_links: [
   {
    state: {
     title: 'Elon Musk sells $3.6bn of Tesla shares'
    },
    url: '/news/business-63981767'
   },
   {
    state: {
     title: 'he sold another 22 million shares'
    },
    url: '/news/business-63981767'
   },
   {
    state: {
     title: "Elon Musk no longer world's richest man"
    },
    url: '/news/business-63963239'
   },
   {
    state: {
     title: 'Twitter Files spark debate about ‘blacklisting’'
    },
    url: '/news/technology-63963779'
   }
  ],
  related_topics: [
   {
    title: 'Elon Musk',
    url: '/news/topics/c302m85q53mt'
   },
   {
    title: 'Twitter',
    url: '/news/topics/cmj34zmwx51t'
   }
  ],
  timestamp: s'2022-12-15T02:37:07.000Z',
  url: '/news/world-us-canada-63978323'
 },
 title: 'Elon Musk taking legal action over Twitter account that tracks his private jet'
}

```

## Links

Below are examples of links created between articles. The following contains examples of incoming and outgoing links. Incoming links are links that point to the article, while outgoing links are links that the article points to.

The data is in JSON format and contains the following fields:

- `id`: The ID of the link.
- `in`: The article that the link points to.
- `out`: The article that the link originates from.

### Example incoming links from the `links_to` table

```json
{
    {
        id: links_to:dbeqgg70c1ubj7ruatm8,
        in: pages:⟨/news/world-us-canada-68223602⟩,
        out: pages:⟨/news/world-us-canada-63978323⟩
    },
    {
        id: links_to:f7i6onizemkg6683b8wl,
        in: pages:⟨/news/world-us-canada-68248168⟩,
        out: pages:⟨/news/world-us-canada-63978323⟩
    }
}
```

### Example outgoing links from the `links_to` table

```json
{
    {
        id: links_to:2cu6uqxhufog611hjeak,
        in: pages:⟨/news/world-us-canada-63978323⟩,
        out: pages:⟨/news/technology-63963779⟩
    }
    {
        id: links_to:pge2x35rq42cojvji30w,
        in: pages:⟨/news/world-us-canada-63978323⟩,
        out: pages:⟨/news/business-63963239⟩
    }
}
```

## Topics

Below are examples of topics related to articles. The data is in JSON format and contains the following fields:

- `id`: The ID of the topic.
- `title`: The title of the topic.
- `url`: The URL of the topic.

### Example from the `topics` table

```json
{
    {
        id: topics:Twitter,
        title: 'Twitter',
        url: '/news/topics/cmj34zmwx51t'
    },
    {
        id: topics:⟨Elon Musk⟩,
        title: 'Elon Musk',
        url: '/news/topics/c302m85q53mt'
    }
}
```

## Topic links

Examples of links between articles and topics. The data is in JSON format and contains the following fields:

- `id`: The ID of the link.
- `in`: The article that the link points to.
- `out`: The topic that the link originates from.

### Example from the `has_topic` table

```json

{
    {
        id: has_topic:va0ehofvvz15vn3wizk1,
        in: pages:⟨/news/world-us-canada-63978323⟩,
        out: topics:⟨Elon Musk⟩
    },
    {
        id: has_topic:64lw511hplfgn6xn5yfg,
        in: pages:⟨/news/world-us-canada-63978323⟩,
        out: topics:Twitter
    }
}
