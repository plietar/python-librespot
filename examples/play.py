import sys
from librespot import Session, SpotifyId
import threading

if len(sys.argv) != 4:
  print("Usage: %s USERNAME PASSWORD TRACK" % sys.argv[0])
  sys.exit(1)

username = sys.argv[1]
password = sys.argv[2]
trackid = SpotifyId(sys.argv[3])

print("Connecting ...")
session = Session.connect(username, password).wait()
player = session.player()

print(threading.get_ident())
def print_track(track):
    print(threading.get_ident())
    print("Playing track \"%s\"..." % (track.name(),))

session.get_track(trackid).spawn(print_track)

print("Playing ...")
player.load(trackid).wait()


print("Done")
