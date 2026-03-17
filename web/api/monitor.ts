import "dotenv/config";
import type { VercelRequest, VercelResponse } from "@vercel/node";
import { monitor } from "../packages/operations/dist/src/monitor";

export const config = {
  maxDuration: 60,
};

export default async function handler(req: VercelRequest, res: VercelResponse) {
  if (req.method !== "GET") {
    res.setHeader("Allow", "GET");
    return res.status(405).json({ error: "Method not allowed" });
  }

  const cronSecret = process.env.CRON_SECRET;
  if (!cronSecret) {
    return res.status(503).json({
      error: "Service unavailable",
      message: "CRON_SECRET is not configured",
    });
  }
  const authHeader = req.headers.authorization;
  if (authHeader !== `Bearer ${cronSecret}`) {
    return res.status(401).json({ error: "Unauthorized" });
  }

  try {
    const metrics = await monitor();
    return res.status(200).json({ ok: true, name: metrics.name });
  } catch (error) {
    console.error("Monitor error:", error);
    return res.status(500).json({
      error: "Monitor failed",
      message: "Monitor failed",
    });
  }
}
