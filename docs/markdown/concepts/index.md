---
layout: default
title: Concepts and Architecture
nav_order: 2
has_children: true
permalink: /concepts/
---

# Concepts and Architecture

Snowbridge is a complex project with lots of moving pieces that can interact in various configurations. In order to develop a comprehensive understanding of how things work, this section tries to break down and run through those pieces into different concepts and components that should be easier to understand.

## Layered Architecture

Snowbridge has a layered architecture with a clear seperation between low level bridge functionality, mid level trust functionality and high level application functionality.

You are probably familiar with the conventional [Internet protocol suite (TCP/IP)](https://en.wikipedia.org/wiki/Internet_protocol_suite){:target="_blank"}, our system is a similar simplified version.

You can have communication go from one layer up or down to another layer - for example, between the App Layer and the Bridge Layer, as in the green arrow below.

You also have a layer-specific protocol across a single layer, where components on different chains at the same layer communicate with each other via that protocol - for example, an app-specific protocol between the App Layer on Polkadot and the App Layer on Ethereum, as in the orange arrow below.

![Layered Architecture](/images/layered-architecture.png){: style="max-width: 200%" }

## High-level overview of architecture

The below document gives a high level overview of the architecture, most components and the communication flow across the brige.

![Architecture Diagram](/images/architecture-diagram.png){: style="max-width: 200%" }

> **Note:**
>
> - When trying to understand this architecture, as you go through these docs, it can be valuable to first imagine the project as two simpler one-way bridges that, when combined at the application layer, allow applications to connect them to form 2 way bridging.
