test-with-curl:
	curl --insecure -XPUT "https://localhost:9200/caa/_doc/1" -H 'Content-Type: application/json' -u admin:3VndwCZowsNqXoHFfHo2z2ay974Xf7yFhW -kv  -d' \
	{ \
	"name": "XXX", \
	"price": 29.99, \
	"description": "xxx" \
	}' 

