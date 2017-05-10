import sys
import spotipy
from librespot import Session, SpotifyId

# This example uses librespot to authenticate, but spotipy to fetch metadata

if len(sys.argv) != 4:
  print("Usage: %s CLIENTID USERNAME PASSWORD" % sys.argv[0])
  sys.exit(1)

clientid = sys.argv[1]
username = sys.argv[2]
password = sys.argv[3]
session = Session.connect(username, password).wait()

token = session.web_token(clientid, "playlist-read-private").wait()

sp = spotipy.Spotify(auth=token.access_token())
results = sp.current_user_playlists()
for item in results['items']:
  print(item['name'])
