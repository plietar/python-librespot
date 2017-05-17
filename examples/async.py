import sys
from threading import Event
from librespot import Session, SpotifyId

if len(sys.argv) != 4:
  print("Usage: %s USERNAME PASSWORD ALBUM" % sys.argv[0])
  sys.exit(1)

username = sys.argv[1]
password = sys.argv[2]
albumid = SpotifyId(sys.argv[3])

done = Event()

print("Connecting ...")
session = Session.connect(username, password).wait()
player = session.player()

def play_album(album):
  print("Playing album \"%s\" ..." % album.name())
  album.tracks().add_callback(play_tracks)

def play_tracks(tracks):
  if len(tracks) > 0:
    track = tracks[0]
    tracks = tracks[1:]

    print("Playing track \"%s\" ..." % track.name())

    # Load the current track
    # Play the tail of the list when it is done
    player \
        .load(track.id()) \
        .add_callback(lambda _: play_tracks(tracks))
  else:
    # All the tracks have been played.
    # Signal the main thread
    done.set()

# Load the album metadata, add a callback to start
# playing all of its tracks
session \
    .get_album(albumid) \
    .add_callback(play_album)

done.wait()

