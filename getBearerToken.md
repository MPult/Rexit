# Getting A Bearer Token

It is pretty easy to get a bearer token from Reddit. You just need to follow these steps:

1. Login to Reddit.com
2. Open the Developer Tools (F12) and go to the Network tab.
3. Go to [chat.reddit.com](https://chat.reddit.com)
4. Open any request that has "messages" in the name and find the field Authorization in the Headers tab. Copy the value of that field beginning after the word "Bearer". It should be very long and likely starts with "ey".
5. Use that value as the bearer token when prompted by Rexit.