import useProfileQuery from "@/api/profile.query.tsx";
import { LucideUser2 } from "lucide-react";
import {
  Avatar,
  AvatarFallback,
  AvatarImage,
} from "@/components/ui/avatar.tsx";
import { Skeleton } from "@/components/ui/skeleton.tsx";

const Profile = () => {
  const { data, isPending, isError } = useProfileQuery();

  if (isPending) {
    return <Skeleton className="border border-border w-40 h-12" />;
  }

  if (isError) {
    return (
      <Skeleton className="border border-destructive/75 bg-destructive/35 animate-none w-40 h-12" />
    );
  }

  return (
    <div className="flex items-center justify-center gap-4 bg-secondary border border-border px-3 py-1 h-12 hover:bg-secondary-foreground/25">
      <div className="flex-1 flex flex-col items-end text-end">
        <h1 className="font-medium leading-none">{data.name}</h1>
        <p className="text-xs text-secondary-foreground leading-none">
          #{data.id}
        </p>
      </div>
      <Avatar className="h-full outline-none after:border-none">
        {!!data.avatar_url && <AvatarImage src={data.avatar_url} />}
        <AvatarFallback>
          <LucideUser2 />
        </AvatarFallback>
      </Avatar>
    </div>
  );
};
export default Profile;
