import sys
from librespot import Session, SpotifyId

if len(sys.argv) != 4:
  print("Usage: %s USERNAME PASSWORD TRACK" % sys.argv[0])
  sys.exit(1)

[_, username, password, trackid] = sys.argv

print("Connecting ...")
session = Session(username, password)
player = session.player()

print("Playing ...")
track = SpotifyId(trackid)
player.play(track)

print("Done")
