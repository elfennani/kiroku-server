import {useQuery} from "@tanstack/react-query";

export interface Media {
    id: number,
    title: string,
    description?: string | null,
    cover?: Image | null,
    banner?: string | null,
    media_type: "ANIME",
    status: MediaStatus,
}

export interface Image {
    thumbnail: string,
    url: string,
    width?: number | null,
    height?: number | null,
}

export interface MediaStatus {
    status?: Status | null,
    progress?: number | null,
    total?: number | null,
}

export type Status = |
    "CURRENT" |
    "COMPLETED" |
    "PLANNED" |
    "REVISITING" |
    "DROPPED" |
    "PAUSED";

const useOngoingMediaQuery = () => {
    return useQuery({
        queryKey: ["ongoing"],
        queryFn: async () => {
            const res = await fetch("/api/media/ongoing");
            const data: Media[] = await res.json()
            return data;
        }
    })
}

export default useOngoingMediaQuery