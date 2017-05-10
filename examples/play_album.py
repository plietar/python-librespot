import sys
from librespot import Session, SpotifyId

if len(sys.argv) != 4:
  print("Usage: %s USERNAME PASSWORD ALBUM" % sys.argv[0])
  sys.exit(1)

username = sys.argv[1]
password = sys.argv[2]
albumid = SpotifyId(sys.argv[3])

print("Connecting ...")
session = Session.connect(username, password).wait()
player = session.player()

album = session.get_album(albumid).wait()
tracks = album.tracks().wait()

print("Playing album \"%s\" (%d tracks)..." % (album.name(), len(tracks)))

for track in tracks:
  print("Playing track \"%s\" ..." % track.name())
  player.load(track.id()).wait()

print("Done")
