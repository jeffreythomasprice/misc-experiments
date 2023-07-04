package main

import (
	"net/http"
	"regexp"
	"strings"

	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

type RouteFunc func(log *zap.Logger, response http.ResponseWriter, request *RouteMatchedRequest) error

type RouteMatchedRequest struct {
	*http.Request
	pathSubmatch []string
}

type RouterBuilder struct {
	routes []*Route
	log    *zap.Logger
}

type logableHttpRequest struct{ *http.Request }

type logableHttpHeader struct{ http.Header }

func NewRouterBuilder(log *zap.Logger) *RouterBuilder {
	return &RouterBuilder{
		routes: nil,
		log:    log.Named("router"),
	}
}

func (builder *RouterBuilder) Build() http.HandlerFunc {
	type match struct {
		request *RouteMatchedRequest
		route   *Route
	}

	routes := make([]*Route, len(builder.routes))
	copy(routes, builder.routes)

	// TODO make possibleMatches a shared thing so these become thread safe?
	possibleMatches := make([]match, 0)

	return func(response http.ResponseWriter, request *http.Request) {
		log := builder.log.With(zap.Object("request", logableHttpRequest{request}))
		log.Debug("received request")

		// clear but keep any allocated capacity
		possibleMatches = possibleMatches[:0]

		// find all the ones that match
		for _, route := range routes {
			m := route.Match(request)
			if m != nil {
				possibleMatches = append(possibleMatches, match{m, route})
			}
		}
		if len(possibleMatches) == 0 {
			log.Debug("no matched route")
			if err := RespondWithError(response, request, http.StatusNotFound); err != nil {
				log.Error("error responding to request", zap.Error(err))
			}
			return
		}
		// no smart sorting of matches, first one wins
		// if you need multiple matchers with overlapping regexp you've probably made a silly api
		bestMatch := possibleMatches[0]

		var routeDescription strings.Builder
		_, _ = routeDescription.WriteString(bestMatch.route.method)
		if bestMatch.route.pathRegexp == nil {
			_, _ = routeDescription.WriteString(" <any path>")
		} else {
			_, _ = routeDescription.WriteString(" ")
			_, _ = routeDescription.WriteString(bestMatch.route.pathRegexp.String())
		}
		log = log.With(zap.String("route", routeDescription.String()))
		log.Debug("matched route")

		if err := bestMatch.route.f(log, response, bestMatch.request); err != nil {
			log.Error("error trying to respond", zap.Error(err))
			if err := RespondWithError(response, request, http.StatusInternalServerError); err != nil {
				log.Error("error trying to write generic error response to a previous error", zap.Error(err))
			}
		}
	}
}

func (builder *RouterBuilder) Add(route *Route) *RouterBuilder {
	builder.routes = append(builder.routes, route)
	return builder
}

func (builder *RouterBuilder) Get(pathRegexp *regexp.Regexp, f RouteFunc) *RouterBuilder {
	return builder.Add(NewRoute(f).
		Method("GET").
		PathRegex(pathRegexp))
}

func (builder *RouterBuilder) Post(pathRegexp *regexp.Regexp, f RouteFunc) *RouterBuilder {
	return builder.Add(NewRoute(f).
		Method("POST").
		PathRegex(pathRegexp))
}

func (builder *RouterBuilder) Put(pathRegexp *regexp.Regexp, f RouteFunc) *RouterBuilder {
	return builder.Add(NewRoute(f).
		Method("PUT").
		PathRegex(pathRegexp))
}

func (builder *RouterBuilder) Delete(pathRegexp *regexp.Regexp, f RouteFunc) *RouterBuilder {
	return builder.Add(NewRoute(f).
		Method("DELETE").
		PathRegex(pathRegexp))
}

func (builder *RouterBuilder) Options(pathRegexp *regexp.Regexp, f RouteFunc) *RouterBuilder {
	return builder.Add(NewRoute(f).
		Method("OPTIONS").
		PathRegex(pathRegexp))
}

func (builder *RouterBuilder) Patch(pathRegexp *regexp.Regexp, f RouteFunc) *RouterBuilder {
	return builder.Add(NewRoute(f).
		Method("PATCH").
		PathRegex(pathRegexp))
}

type Route struct {
	f          RouteFunc
	method     string
	pathRegexp *regexp.Regexp
}

func NewRoute(f RouteFunc) *Route {
	return &Route{
		f:      f,
		method: "GET",
	}
}

func (route *Route) Method(method string) *Route {
	route.method = strings.ToUpper(method)
	return route
}

func (route *Route) PathRegex(pathRegexp *regexp.Regexp) *Route {
	route.pathRegexp = pathRegexp
	return route
}

func (route *Route) Match(request *http.Request) *RouteMatchedRequest {
	if route.method != strings.ToUpper(request.Method) {
		return nil
	}
	var pathSubmatch []string
	if route.pathRegexp != nil {
		pathSubmatch = route.pathRegexp.FindStringSubmatch(request.URL.Path)
		if pathSubmatch == nil {
			return nil
		}
	}
	return &RouteMatchedRequest{
		request,
		pathSubmatch,
	}
}

// zap.ObjectMarshaler
func (r logableHttpRequest) MarshalLogObject(enc zapcore.ObjectEncoder) error {
	enc.AddString("url", r.URL.String())
	enc.AddString("method", r.Method)
	enc.AddObject("headers", logableHttpHeader{r.Header})
	return nil
}

// zap.ObjectMarshaler
func (h logableHttpHeader) MarshalLogObject(enc zapcore.ObjectEncoder) error {
	for name, values := range h.Header {
		if len(values) == 1 {
			enc.AddString(name, values[0])
		} else {
			zap.Strings(name, values).AddTo(enc)
		}
	}
	return nil
}
