import useProfileQuery from "@/api/profile.query.tsx";
import {LucideLoader2, LucideTriangleAlert, LucideUser2} from "lucide-react";
import {Alert, AlertDescription, AlertTitle} from "@/components/ui/alert.tsx";
import {Avatar, AvatarFallback, AvatarImage} from "@/components/ui/avatar.tsx";

const Profile = () => {
    const {data, isPending, isError, error} = useProfileQuery();

    if (isPending) {
        return (
            <div className="w-full flex items-center justify-center h-64">
                <LucideLoader2 className="animate-spin"/>
            </div>
        )
    }

    if (isError) {
        return <Alert>
            <LucideTriangleAlert/>
            <AlertTitle>Failed to fetch profile!</AlertTitle>
            <AlertDescription>{error.message}</AlertDescription>
        </Alert>
    }

    return (
        <div className="flex items-center gap-4">
            <Avatar className="size-12">
                {!!data.avatar_url && <AvatarImage src={data.avatar_url}/>}
                <AvatarFallback>
                    <LucideUser2/>
                </AvatarFallback>
            </Avatar>
            <div className="flex-1 flex flex-col gap-2">
                <h1 className="text-lg font-bold">{data.name}</h1>
                <p className="text-sm text-secondary-foreground">{data.description}</p>
            </div>
        </div>
    );
};
export default Profile;
