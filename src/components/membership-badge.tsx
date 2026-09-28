import gold from "@/assets/badges/gold.png";
import platinum from "@/assets/badges/platinum.png";
import diamond from "@/assets/badges/diamond.png";
import elite from "@/assets/badges/elite.png";
import { membershipLevelName } from "@/lib/membership-levels";
import { cn } from "@/lib/utils";

// Client-supplied badge art (128px copies of assets/*_tier.png), rank 1..4.
// Always decorative: every use shows the level name as text beside it, so
// the image carries no meaning a screen reader would miss.
const BADGES = [gold, platinum, diamond, elite];

function MembershipBadge({
  rank,
  size,
  className,
}: {
  rank: number;
  size: number;
  className?: string;
}) {
  const src = BADGES[rank - 1];
  if (!src) return null;
  return (
    <img
      src={src}
      alt=""
      title={`${membershipLevelName(rank)} membership`}
      width={size}
      height={size}
      className={cn("shrink-0 object-contain", className)}
    />
  );
}

// Badge + name after a member's name in lists (search results, slab drill-down).
function MemberLevelTag({ rank }: { rank: number }) {
  if (!BADGES[rank - 1]) return null;
  return (
    <span className="inline-flex shrink-0 items-center gap-1">
      <MembershipBadge rank={rank} size={22} />
      <span className="text-[11px] font-semibold text-muted-text">{membershipLevelName(rank)}</span>
    </span>
  );
}

export { MembershipBadge, MemberLevelTag };
