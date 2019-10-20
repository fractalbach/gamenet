import argparse
import json
from pathlib import Path
import sys
import typing as ty

import matplotlib.pyplot as plt
import networkx as nx

USIZE_MAX = 2 ** 64 - 1


commands: ty.Dict[str, ty.Callable[[], None]] = {}


def expose(
        f: ty.Callable[[ty.Sequence[str]], None]
) -> ty.Callable[[ty.Sequence[str]], None]:
    commands[f.__name__] = f
    return f


class Vec2(ty.NamedTuple):
    x: ty.Union[float, int]
    y: ty.Union[float, int]


class Node:
    def __init__(self, **kwargs):

        def fn(i):
            return i != USIZE_MAX

        self.i = kwargs['i']
        self.indices = Vec2(**kwargs['indices'])
        self.uv = Vec2(**kwargs['uv'])
        self.h = kwargs['h']
        self.neighbors = list(filter(fn, kwargs['neighbors']))
        self.inlets = list(filter(fn, kwargs['inlets']))
        self.outlet = kwargs['outlet'] if kwargs['outlet'] else None
        self.direction = Vec2(**kwargs['direction'])
        self.fork_angle = kwargs['fork_angle']
        self.strahler = kwargs['strahler']


@expose
def graph(args: ty.Sequence[str]) -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument('path')
    path = Path(parser.parse_args(args).path)

    print('Loading...')
    with path.open() as f:
        data = json.load(f)

    graph = nx.Graph()

    nodes = [Node(**d) for d in data['graph']]

    for i, node in enumerate(nodes):
        inlets = node.inlets
        if node.h < 0 and not inlets:
            continue

        graph.add_node(node.i, pos=(node.uv.x, node.uv.y))

        print(f'Adding nodes... {i / len(data["graph"]) * 100:.2f}%\r', end='')
    print(f'Adding nodes... 100.00%!')

    for i, node in enumerate(nodes):
        for inlet in node.inlets:
            if inlet != USIZE_MAX:
                graph.add_edge(
                    node.i, inlet, color='b', width=nodes[inlet].strahler
                )
        print(f'Adding edges... {i / len(data["graph"]) * 100:.2f}%\r', end='')
    print(f'Adding edges... 100.00%!')

    edges = graph.edges()
    nx.draw(
        graph,
        pos=nx.get_node_attributes(graph, 'pos'),
        edge_color=[graph[u][v]['color'] for u, v in edges],
        width=[graph[u][v]['width'] for u, v in edges],
        node_size=2,
    )
    plt.axis('equal')
    plt.show()


def main():
    """
    Display river graph info.
    :return:
    """
    parser = argparse.ArgumentParser()
    parser.add_argument('command', choices=commands.keys())
    cmd = parser.parse_args(sys.argv[1:2]).command
    commands[cmd](sys.argv[2:])


if __name__ == '__main__':
    main()
