import "dotenv/config"
import Fastify from "fastify"
import { monitor } from "./monitor"

const fastify = Fastify({ logger: true })

fastify.setErrorHandler((error, request, reply) => {
    fastify.log.error(error)
    reply.status(500).send({ ok: false })
})

fastify.register(import("@fastify/rate-limit"), {
    global: true,
    max: 2,
    timeWindow: 5000,
})

fastify.get("/monitor", async (request, reply) => {
    let metrics = await monitor()
    let message = JSON.stringify(
        metrics,
        (key, value) => (typeof value === "bigint" ? value.toString() : value),
        2
    )
    reply.send(message)
})

fastify.get("/hello", (request, reply) => {
    reply.send({ hello: "world" })
})

fastify.listen({ port: 3000 })
