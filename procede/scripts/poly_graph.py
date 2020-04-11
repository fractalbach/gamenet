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


def discover_poly(path: Path):
    with path.open() as f:
        data = json.load(f)

    def recurse(entry: ty.Dict[str, ty.Any]):
        # Recognize polygons by their inner member 'exterior'.
        for k, v in entry.items():
            if k == 'exterior':
                yield Poly(v)
            else:
                if isinstance(v, dict):
                    yield from recurse(v)

    yield from recurse(data)


def graph_poly(path: Path):
    graph = nx.Graph()

    for poly in discover_poly(path):
        poly.add(graph)

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
