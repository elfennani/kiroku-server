import { useQuery } from "@tanstack/react-query";
import type { Image, MediaStatus } from "@/api/ongoing.query.ts";

interface ProcessedMedia {
  id: string;
  episode: number;
  duration: number;
  title?: string | null;
  thumbnail?: string | null;
}

interface MediaDetails {
  id: number;
  title: string;
  description?: string | null;
  cover?: Image | null;
  banner?: string | null;
  status: MediaStatus;
  episodes: ProcessedMedia[];
}

const useMediaQuery = (mediaId: number) => {
  return useQuery({
    queryKey: ["media", mediaId],
    queryFn: async () => {
      const res = await fetch(`/api/media/${mediaId}`);
      const data: MediaDetails = await res.json();

      return data;
    },
  });
};

export default useMediaQuery;
