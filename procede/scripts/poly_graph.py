import argparse
import json
from pathlib import Path
import typing as ty
from uuid import uuid4

import matplotlib.pyplot as plt
import networkx as nx


class Poly:
    def __init__(self, exterior: ty.List[dict]) -> None:
        # Accepts a list of dictionaries with x and y keys.
        self.exterior = exterior

    def add(self, graph: nx.Graph) -> None:
        points = self.exterior[:-1]
        ids = [uuid4() for _ in points]
        for uuid, pos in zip(ids, points):
            graph.add_node(uuid, pos=(pos['x'], pos['y']))
        for i, a in enumerate(ids + [ids[0]]):
            b = ids[(i + 1) % len(ids)]
            graph.add_edge(a, b)


class Point:
    def __init__(self, data: dict) -> None:
        self.x = data['x']
        self.y = data['y']

    def add(self, graph: nx.Graph) -> None:
        graph.add_node(uuid4(), pos=(self.x, self.y))


def discover_poly(path: Path):
    with path.open() as f:
        data = json.load(f)

    def recurse(obj: ty.Any):
        if isinstance(obj, dict):
            for k, v in obj.items():
                # Recognize polygons by their inner member 'exterior'.
                if k == 'exterior' and isinstance(v, list):
                    yield Poly(v)
                elif k == 'y' and 'x' in obj:
                    yield Point(obj)
                    break
                else:
                    yield from recurse(v)
        elif isinstance(obj, list):
            for item in obj:
                yield from recurse(item)

    yield from recurse(data)


def graph_poly(path: Path):
    graph = nx.Graph()

    for obj in discover_poly(path):
        obj.add(graph)

    nx.draw(
        graph,
        pos=nx.get_node_attributes(graph, 'pos'),
        node_size=2,
    )
    plt.axis('equal')
    plt.show()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('path', type=Path)
    args = parser.parse_args()
    graph_poly(args.path)


if __name__ == '__main__':
    main()
