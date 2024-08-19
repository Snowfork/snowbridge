import "dotenv/config"
import cron from "node-cron"
import { monitor } from "./monitor"

cron.schedule("*/30 * * * *", monitor)