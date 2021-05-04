To start receiving the quotes.
1 Open orderbook_server and enter "cargo run", the server will be listeneing
2 open client_orderbook and enter again "cargo run", the client makes one stream request.
3 the quotes will be shown at the console.

When you run the server the OrderbookAggregator will start the all the exchanges tasks, and set up all the channels.
the stream connection with binance can be done in one step through the url, while in bistamp we need two steps, first a request connection to the base url
and then a subscription message.  

bitstamp streams enough data so there is no need for sync (the data enough book depth), but in binance case we need to get snapshots to sync incoming small books with the big one.

please check the Aggregated_ob_schema.pdf to check a the project flow.



