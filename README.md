# Overview
Inspired by the laws outlined in the [Frugal Architecture](https://thefrugalarchitect.com/laws/make-cost-a-non-functional-requirement.html), this project attempts to support these laws, specifically I, II and IV - treating cost as a non-functional requirement and improving cost observability. 

## Context
The cost of infrastructure in isolation has no context, especially over time. Does it make sense that my cloud costs have increased, decreased, or even plateaued relative to previous months? The first phase of this program tries to provide some context for lambda based services. 

For a given service, the program will collect:
- total monthly costs
- total monthly invocations
- the average latency duration
- and calculate cost per requests - specifically: (monthly costs / monthly invocations) * 100M.

With these metrics plotted over time (months) it can highlight anomalies and more importantly that the chosen architecture is no scaling and/or [is no longer exploiting economies of scale (law II)](https://thefrugalarchitect.com/laws/systems-that-last-align-cost-to-business.html) 

For example:
- an increase in invocations, but no decrease in cost per requests could indicate that economies of scale is not being exploited.
- an increase in cost per requests, but no increase in invocations or duration (key factor for lambda based workloads) could indicate an anomaly (e.g. configuration change).
- an increase in duration should result in an increase in cost per requests, and therefore highlights the impact and explains why cloud costs have increased.
