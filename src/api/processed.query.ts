import { useQuery } from "@tanstack/react-query";

interface ProcessedMedia {
  id: string;
  episode: number;
  duration: number;
}

const useProcessedMediaQuery = (mediaId: number) => {
  return useQuery({
    queryKey: ["media", "processed", mediaId],
    queryFn: async () => {
      const res = await fetch(`/api/media/${mediaId}/processed`);
      const data: ProcessedMedia[] = await res.json();

      return data;
    },
  });
};

export default useProcessedMediaQuery;
