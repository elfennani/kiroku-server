import { Link, useParams } from "react-router";
import {
  LucideArrowRight,
  LucideImageOff,
  LucideLoader2,
  LucideTriangleAlert,
} from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert.tsx";
import useMediaQuery from "@/api/processed.query.ts";

const MediaRoute = () => {
  const { id } = useParams();
  const { data, isPending, isError, error } = useMediaQuery(Number(id));

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

  return (
    <div className="max-lg:pb-8">
      <h2 className="text-secondary-foreground border-b pb-2 font-medium text-lg max-lg:hidden">
        {data.title}
      </h2>

      <div className="flex flex-col gap-4 w-full">
        <header className="relative mb-16 max-sm:-mx-6">
          {!!data.banner && (
            <img
              src={data.banner}
              alt=""
              className="w-full object-cover h-48 md:aspect-4/1"
            />
          )}
          <div className="absolute inset-0 from-background/30 to-background bg-linear-to-b pointer-events-none" />
        </header>
        <img
          src={data.cover?.thumbnail}
          alt=""
          className="aspect-[0.69] w-24 md:w-32 relative -mt-48 left-4 lg:left-8"
        />
        <div className="px-4 lg:px-8 space-y-8">
          <div>
            <span className="text-sm uppercase text-secondary-foreground tracking-wider">
              ANIME
            </span>
            <h1 className="text-xl md:text-2xl lg:text-3xl font-medium line-clamp-2">
              {data.title}
            </h1>
          </div>

          <div>
            <h3 className="text-sm font-medium uppercase text-secondary-foreground tracking-wider mb-2">
              Description
            </h3>
            {!!data.description && (
              <p
                dangerouslySetInnerHTML={{ __html: data.description }}
                className="line-clamp-3 text-xs md:text-sm leading-relaxed"
              />
            )}
          </div>
        </div>
        <div className="mt-4">
          <h3 className="text-sm font-medium uppercase text-secondary-foreground tracking-wider mb-2 mx-4 lg:mx-8">
            Episodes
          </h3>
          {data.episodes.map((media) => (
            <Link
              to={`/episode/${media.id}`}
              key={media.id}
              className="flex px-4 lg:px-8 py-2 group gap-4 items-center cursor-pointer hover:bg-secondary-foreground/10"
            >
              <div className="w-24 md:w-32 aspect-video bg-secondary flex items-center justify-center text-secondary-foreground">
                {media.thumbnail ? (
                  <img
                    src={media.thumbnail}
                    alt=""
                    className="size-full object-contain"
                  />
                ) : (
                  <LucideImageOff className="max-md:size-4" />
                )}
              </div>
              <div className="flex flex-col flex-1">
                <h2 className="font-medium group-hover:underline underline-offset-4 max-md:text-sm">
                  {media.title ?? <>Episode {media.episode}</>}
                </h2>
                <p className="text-secondary-foreground text-xs md:text-sm">
                  {Math.round(media.duration / (1000 * 1000 * 60))} mins
                </p>
              </div>
              <LucideArrowRight className="max-md:size-4" />
            </Link>
          ))}
        </div>
      </div>
    </div>
  );
};

const Layout = ({ children }: { children: React.ReactNode }) => {
  return <div className="max-w-xl mx-auto">{children}</div>;
};

export default MediaRoute;
