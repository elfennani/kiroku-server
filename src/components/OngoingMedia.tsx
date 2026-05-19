import useOngoingMediaQuery from "@/api/ongoing.query.ts";
import { LucideLoader2, LucideTriangleAlert } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert.tsx";
import { Link } from "react-router";

const OngoingMedia = () => {
  const { data, isPending, isError, error } = useOngoingMediaQuery();

  if (isPending) {
    return (
      <div className="w-full flex items-center justify-center h-64">
        <LucideLoader2 className="animate-spin" />
      </div>
    );
  }

  if (isError) {
    return (
      <Alert>
        <LucideTriangleAlert />
        <AlertTitle>Failed to fetch ongoing media!</AlertTitle>
        <AlertDescription>{error.message}</AlertDescription>
      </Alert>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <h2 className="text-secondary-foreground border-b pb-2 font-medium text-lg">
        Anime
      </h2>

      {data.map((medium) => (
        <Link
          to={`/media/${medium.id}`}
          key={medium.id}
          className="hover:scale-[102%] group transition-transform duration-300 flex gap-4 items-stretch"
        >
          <img
            className="w-24 aspect-[0.69] object-cover"
            src={medium.cover?.thumbnail}
            alt={medium.title}
          />
          <div className="py-2 flex flex-col justify-between flex-1">
            <div>
              <span className="text-sm text-secondary-foreground font-semibold tracking-wide">
                {medium.status.status}
              </span>
              <h2 className="text-lg line-clamp-2 group-hover:underline underline-offset-4">
                {medium.title}
              </h2>
            </div>
            <div>
              <div className="flex justify-between items-center text-sm text-foreground/75">
                <span>Progress</span>
                <span>
                  <span className="text-xl font-bold text-foreground mr-0.5">
                    {medium.status.progress}
                  </span>
                  /{medium.status.total}
                </span>
              </div>
              <div className="bg-primary-foreground h-1">
                <div
                  className="bg-primary h-full relative after:absolute after:inset-0 after:bg-primary after:blur-md"
                  style={{
                    width: `${((medium.status.progress ?? 0) / (medium.status.total ?? 1)) * 100}%`,
                  }}
                />
              </div>
            </div>
          </div>
        </Link>
      ))}
    </div>
  );
};
export default OngoingMedia;
