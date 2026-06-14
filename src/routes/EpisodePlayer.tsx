import { useEffect, useRef, useState } from "react";
import { Link, useParams } from "react-router";
import Hls from "hls.js";
// @ts-ignore
import Plyr from "plyr";
import "plyr/dist/plyr.css";
import {
  LucideArrowLeft,
  LucideLoader2,
  LucideTriangleAlert,
} from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert.tsx";
import $api from "@/api/api-client.ts";
import type { components } from "@/api/schema";

type AudioTrackOption = {
  index: number;
  label: string;
};

type NativeVideoElement = HTMLVideoElement & {
  audioTracks?: {
    length: number;
    [index: number]: {
      label?: string;
      language?: string;
    };
  };
};

const playerOptions = {
  captions: {
    active: true,
    update: true,
    language: "en",
  },
  settings: ["captions", "quality", "speed"],
};

type Episode = components["schemas"]["Episode"];

const EpisodePlayerRoute = () => {
  const { episodeId } = useParams();
  const { data, isPending, isError, error } = $api.useQuery(
    "get",
    "/episodes/{id}",
    {
      params: {
        path: {
          id: episodeId!!,
        },
      },
    },
    { select: ({ data }) => data },
  );

  if (isPending) {
    return (
      <Layout>
        <div className="w-full flex items-center justify-center h-64">
          <LucideLoader2 className="animate-spin" />
        </div>
      </Layout>
    );
  }

  if (isError) {
    return (
      <Layout>
        <Alert>
          <LucideTriangleAlert />
          <AlertTitle>Failed to fetch processed media!</AlertTitle>
          <AlertDescription>{error.message}</AlertDescription>
        </Alert>
      </Layout>
    );
  }

  return <EpisodePlayer episode={data} />;
};

const EpisodePlayer = ({ episode }: { episode: Episode }) => {
  // const { episodeId } = useParams();
  const videoRef = useRef<HTMLVideoElement>(null);
  const hlsRef = useRef<Hls | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [audioTracks, setAudioTracks] = useState<AudioTrackOption[]>([]);
  const [selectedAudioTrack, setSelectedAudioTrack] = useState(0);
  const [isNativeAudioSupported, setIsNativeAudioSupported] = useState(false);

  useEffect(() => {
    if (!videoRef.current) {
      return;
    }

    const video = videoRef.current;
    const nativeCleanup: Array<() => void> = [];
    let hls: Hls | null = null;
    let player: Plyr | null = null;
    let disposed = false;

    const updateAudioTracksFromHls = () => {
      if (!hls) {
        return;
      }

      const tracks = hls.audioTracks.map((track, index) => ({
        index,
        label: track.name || track.lang || `Track ${index + 1}`,
      }));

      setAudioTracks(tracks);
      setSelectedAudioTrack(hls.audioTrack >= 0 ? hls.audioTrack : 0);
    };

    const updateNativeAudioTracks = () => {
      const tracks = (video as NativeVideoElement).audioTracks;

      if (!tracks || !tracks.length) {
        setAudioTracks([]);
        setSelectedAudioTrack(0);
        return;
      }

      setAudioTracks(
        Array.from({ length: tracks.length }, (_, index) => {
          const track = tracks[index];

          return {
            index,
            label: track.label || track.language || `Track ${index + 1}`,
          };
        }),
      );
      setSelectedAudioTrack(0);
    };

    const fail = (message: string) => {
      if (!disposed) {
        setError(message);
        setIsLoading(false);
      }
    };

    if (Hls.isSupported()) {
      hls = new Hls();
      hlsRef.current = hls;
      const currentHls = hls;
      hls.loadSource(episode.url);
      hls.attachMedia(video);

      currentHls.on(Hls.Events.MANIFEST_PARSED, () => {
        if (disposed) {
          return;
        }

        const qualities = Array.from(
          new Set(
            currentHls.levels
              .map((level) => level.height)
              .filter((height): height is number => height > 0) ?? [],
          ),
        );

        const plyrOptions = qualities.length
          ? {
              ...playerOptions,
              quality: {
                default: qualities[0],
                options: qualities,
                forced: true,
                onChange: (quality: number) => {
                  currentHls.levels.forEach((level, index) => {
                    if (level.height === quality) {
                      currentHls.currentLevel = index;
                    }
                  });
                },
              },
            }
          : playerOptions;

        player = new Plyr(video, plyrOptions);

        currentHls.subtitleTrack = 0;
        updateAudioTracksFromHls();
        currentHls.on(
          Hls.Events.AUDIO_TRACKS_UPDATED,
          updateAudioTracksFromHls,
        );
        setIsLoading(false);
      });

      currentHls.on(Hls.Events.ERROR, (_, data) => {
        if (data.fatal) {
          fail("Failed to load episode stream.");
        }
      });
    } else if (video.canPlayType("application/vnd.apple.mpegurl")) {
      setIsNativeAudioSupported(true);
      video.src = episode.url;
      player = new Plyr(video, playerOptions);

      const handleLoadedMetadata = () => {
        updateNativeAudioTracks();
        setIsLoading(false);
      };

      const handleError = () => fail("Failed to load episode stream.");

      video.addEventListener("loadedmetadata", handleLoadedMetadata);
      video.addEventListener("error", handleError);
      nativeCleanup.push(() =>
        video.removeEventListener("loadedmetadata", handleLoadedMetadata),
      );
      nativeCleanup.push(() => video.removeEventListener("error", handleError));
    } else {
      fail("This browser does not support HLS playback.");
    }

    return () => {
      disposed = true;
      hls?.destroy();
      hlsRef.current = null;
      player?.destroy();
      nativeCleanup.forEach((cleanup) => cleanup());
    };
  }, [episode]);

  if (error) {
    return (
      <Layout>
        <Alert>
          <LucideTriangleAlert />
          <AlertTitle>Failed to load episode</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      </Layout>
    );
  }

  return (
    <Layout>
      <div className="flex items-center gap-3 text-secondary-foreground">
        <Link
          to={`/media/${episode.media.id}`}
          className="inline-flex items-center gap-2 hover:text-foreground"
        >
          <LucideArrowLeft className="size-4" />
          Back
        </Link>
        <span className="text-xs uppercase tracking-wider">
          Episode {episode.number}
        </span>
      </div>

      <div className="space-y-4">
        <div className="bg-black rounded-xl overflow-hidden border border-border">
          <video
            ref={videoRef}
            controls
            playsInline
            className="w-full aspect-video"
          />
        </div>

        <div className="flex flex-wrap items-center gap-3 text-sm">
          <label htmlFor="audioSelector" className="text-secondary-foreground">
            Audio
          </label>
          <select
            id="audioSelector"
            aria-label="Audio tracks"
            className="min-w-44 px-3 py-2 bg-secondary border border-border"
            value={selectedAudioTrack}
            disabled={!audioTracks.length || isNativeAudioSupported}
            onChange={(event) => {
              const nextTrack = Number(event.target.value);
              setSelectedAudioTrack(nextTrack);

              if (Hls.isSupported()) {
                if (hlsRef.current && !Number.isNaN(nextTrack)) {
                  hlsRef.current.audioTrack = nextTrack;
                }
                return;
              }
            }}
          >
            {audioTracks.length ? (
              audioTracks.map((track) => (
                <option key={track.index} value={track.index}>
                  {track.label}
                </option>
              ))
            ) : (
              <option value={0}>Default</option>
            )}
          </select>

          {isLoading && (
            <span className="inline-flex items-center gap-2 text-secondary-foreground">
              <LucideLoader2 className="size-4 animate-spin" />
              Loading player
            </span>
          )}
        </div>
      </div>
    </Layout>
  );
};

const Layout = ({ children }: { children: React.ReactNode }) => {
  return <div className="max-w-5xl mx-auto space-y-4">{children}</div>;
};

export default EpisodePlayerRoute;
