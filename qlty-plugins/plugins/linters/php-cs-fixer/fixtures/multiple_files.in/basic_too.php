<?php declare(strict_types = 1);

class HelloWorld
{
	private $b;

	private $a;


	public function sayHello(DateTimeImutable $date): void
	{
		$c=(ARRAY)array(1,2);
		echo 'Hello, ' . $date->format('j. n. Y');
	}
}
