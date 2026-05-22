import { Link, Outlet, useLocation } from "react-router";
import { LucideSearch } from "lucide-react";
import Profile from "@/components/Profile.tsx";
import OngoingMedia from "@/components/OngoingMedia.tsx";
import { cn } from "@/lib/utils.ts";

const HomeLayout = () => {
  const path = useLocation();
  const isHome = path.pathname == "/";

  return (
    <div className="container px-6 pt-6 md:pt-12 mx-auto flex flex-col gap-y-6 md:gap-y-12 min-h-svh">
      <header className="w-full flex items-center gap-4">
        <Link
          to={"/"}
          className="text-4xl sm:text-5xl font-thin flex-1 hover:underline underline-offset-8"
        >
          Kiroku
        </Link>
        <div className="relative text-secondary-foreground max-lg:hidden">
          <input
            type="text"
            placeholder="Search..."
            className="w-sm h-12 pl-10 pr-4 outline-none peer focus:border-primary bg-secondary text-foreground border border-border placeholder-secondary-foreground"
          />
          <LucideSearch className="absolute left-3 top-1/2 peer-focus:stroke-primary -translate-y-1/2 size-5 stroke-secondary-foreground" />
        </div>
        <Profile />
      </header>
      <div className="relative text-secondary-foreground lg:hidden">
        <input
          type="text"
          placeholder="Search..."
          className="w-full h-12 pl-10 pr-4 outline-none peer focus:border-primary bg-secondary text-foreground border border-border placeholder-secondary-foreground"
        />
        <LucideSearch className="absolute left-3 top-1/2 peer-focus:stroke-primary -translate-y-1/2 size-5 stroke-secondary-foreground" />
      </div>

      <div
        className={cn(
          "grid lg:grid-cols-3 flex-1 gap-4 lg:gap-12",
          isHome && "lg:grid-cols-3",
        )}
      >
        <div className={cn(!isHome && "max-lg:hidden")}>
          <OngoingMedia />
        </div>
        <div className="lg:col-span-2">
          <Outlet />
        </div>
      </div>
    </div>
  );
};
export default HomeLayout;
