import sys
from librespot import Session, SpotifyId

if len(sys.argv) != 4:
  print("Usage: %s USERNAME PASSWORD TRACK" % sys.argv[0])
  sys.exit(1)

username = sys.argv[1]
password = sys.argv[2]
trackid = SpotifyId(sys.argv[3])

print("Connecting ...")
session = Session.connect(username, password).wait()
player = session.player()

print("Playing ...")
player.load(trackid).wait()

print("Done")
