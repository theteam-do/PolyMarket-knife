import json
import urllib.request

url = "https://gamma-api.polymarket.com/events?closed=false&active=true&limit=2"
req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
response = urllib.request.urlopen(req)
data = json.loads(response.read())

for event in data:
    for market in event.get('markets', []):
        print("Q:", market.get('question'))
        print("Outcomes:", market.get('outcomes'))
        print("Prices:", market.get('outcomePrices'))
        print("Token IDs:", market.get('clobTokenIds'))
        print("---")
