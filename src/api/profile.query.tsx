import {useQuery} from "@tanstack/react-query";

export interface Profile {
    id: number;
    name: string;
    avatar_url?: string | null,
    banner_url?: string | null,
    description?: string | null,
}

const useProfileQuery = () => {
    return useQuery({
        queryKey: ["profile"],
        queryFn: async () => {
            const res = await fetch("/api/user/me");
            const data: Profile = await res.json()
            return data;
        }
    });
}

export default useProfileQuery;